#![cfg(unix)]

use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread::sleep,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn unique_dir() -> PathBuf {
    let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "autopilot-e2e-{}-{}-{}",
        std::process::id(),
        id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

struct Daemon {
    child: Child,
    port: u16,
    dir: PathBuf,
}

impl Daemon {
    fn start(jobs: &[(&str, Value)]) -> Self {
        let dir = unique_dir();
        fs::create_dir_all(dir.join("jobs")).expect("create jobs dir");
        for (file, body) in jobs {
            fs::write(
                dir.join("jobs").join(file),
                serde_json::to_vec(body).unwrap(),
            )
            .expect("write job");
        }

        let port = free_port();
        let child = Command::new(env!("CARGO_BIN_EXE_autopilot-rs"))
            .arg("--config-path")
            .arg(&dir)
            .args(["serve", "--api"])
            .env("AUTOPILOT_PORT", port.to_string())
            .env("AUTOPILOT_E2E_VAR", "expected")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn autopilot");

        let daemon = Self { child, port, dir };
        daemon.wait_healthy();
        daemon
    }

    fn wait_healthy(&self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if let Ok((200, _)) = self.request("GET", "/health", None) {
                return;
            }
            sleep(Duration::from_millis(25));
        }
        panic!("daemon did not become healthy");
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<(u16, Value), String> {
        let payload = body
            .map(|value| serde_json::to_string(value).unwrap())
            .unwrap_or_default();
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{payload}",
            payload.len()
        );

        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| e.to_string())?;
        stream
            .write_all(request.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&response);
        let status = text
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|code| code.parse::<u16>().ok())
            .ok_or_else(|| format!("invalid response: {text:?}"))?;
        let body = text
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .unwrap_or("");
        let value = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(body).unwrap_or(Value::Null)
        };

        Ok((status, value))
    }

    fn get(&self, path: &str) -> (u16, Value) {
        self.request("GET", path, None).expect("GET request")
    }

    fn post(&self, path: &str, body: Option<Value>) -> (u16, Value) {
        self.request("POST", path, body.as_ref())
            .expect("POST request")
    }

    fn put(&self, path: &str, body: Value) -> (u16, Value) {
        self.request("PUT", path, Some(&body)).expect("PUT request")
    }

    fn delete(&self, path: &str) -> (u16, Value) {
        self.request("DELETE", path, None).expect("DELETE request")
    }

    fn jobs(&self) -> Vec<Value> {
        let (status, body) = self.get("/jobs");
        assert_eq!(status, 200, "GET /jobs failed");
        body.as_array().cloned().unwrap_or_default()
    }

    fn wait_for_jobs(&self, expected: &[(&str, &str)]) -> Vec<Value> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let jobs = self.jobs();
            let matches = expected.iter().all(|(id, status)| {
                jobs.iter().any(|job| {
                    job.get("id").and_then(Value::as_str) == Some(*id)
                        && job.get("status").and_then(Value::as_str) == Some(*status)
                })
            });
            if matches {
                return jobs;
            }
            if Instant::now() > deadline {
                panic!("jobs did not reach expected states: {jobs:#?}");
            }
            sleep(Duration::from_millis(50));
        }
    }

    fn logs(&self) -> String {
        let mut text = String::new();
        if let Ok(entries) = fs::read_dir(self.dir.join("logs")) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    text.push_str(&content);
                }
            }
        }
        text
    }

    fn shutdown(&mut self) {
        let _ = self.request("POST", "/shutdown", None);
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Ok(Some(_)) = self.child.try_wait() {
                return;
            }
            sleep(Duration::from_millis(25));
        }
        panic!("daemon did not exit after shutdown");
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if !std::thread::panicking() {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

fn command_condition(command: &str) -> Value {
    json!({"type": "command", "condition": {"command": command}})
}

fn job(id: &str, conditions: Vec<Value>, tasks: Vec<Value>) -> Value {
    json!({
        "id": id,
        "name": id,
        "description": "e2e test job",
        "conditions": conditions,
        "tasks": tasks
    })
}

#[test]
fn conditions_run_tasks_and_gate_on_failures() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();
    let marker = |name: &str| dir.join(name).display().to_string();

    let jobs = vec![
        (
            "cmd.jsonc",
            job(
                "job_cmd",
                vec![command_condition("true")],
                vec![json!({"command": format!("touch {}", marker("cmd"))})],
            ),
        ),
        (
            "cmd_output.jsonc",
            job(
                "job_cmd_output",
                vec![json!({"type": "command", "condition": {
                    "command": "echo hello", "check_exit_code": false, "target_output": "hello"
                }})],
                vec![json!({"command": format!("touch {}", marker("cmd_output"))})],
            ),
        ),
        (
            "cmd_fail.jsonc",
            job(
                "job_cmd_fail",
                vec![command_condition("false")],
                vec![json!({"command": format!("touch {}", marker("cmd_fail"))})],
            ),
        ),
        (
            "variable.jsonc",
            job(
                "job_variable",
                vec![json!({"type": "variable", "condition": {
                    "variable": "AUTOPILOT_E2E_VAR", "target": "expected"
                }})],
                vec![json!({"command": format!("touch {}", marker("variable"))})],
            ),
        ),
        (
            "process.jsonc",
            job(
                "job_process",
                vec![json!({"type": "process", "condition": {
                    "process_name": "autopilot-rs", "should_be_running": true
                }})],
                vec![json!({"command": format!("touch {}", marker("process"))})],
            ),
        ),
        (
            "process_absent.jsonc",
            job(
                "job_process_absent",
                vec![json!({"type": "process", "condition": {
                    "process_name": "__e2e_missing_process__", "should_be_running": true
                }})],
                vec![json!({"command": format!("touch {}", marker("process_absent"))})],
            ),
        ),
        (
            "process_not_running.jsonc",
            job(
                "job_process_not_running",
                vec![json!({"type": "process", "condition": {
                    "process_name": "__e2e_missing_process__", "should_be_running": false
                }})],
                vec![json!({"command": format!("touch {}", marker("process_not_running"))})],
            ),
        ),
        (
            "disk.jsonc",
            job(
                "job_disk",
                vec![json!({"type": "diskspace", "condition": {
                    "path": "/", "min_free_gb": 0
                }})],
                vec![json!({"command": format!("touch {}", marker("disk"))})],
            ),
        ),
        (
            "disk_fail.jsonc",
            job(
                "job_disk_fail",
                vec![json!({"type": "diskspace", "condition": {
                    "path": "/", "min_free_gb": 1000000000
                }})],
                vec![json!({"command": format!("touch {}", marker("disk_fail"))})],
            ),
        ),
        (
            "resource.jsonc",
            job(
                "job_resource",
                vec![json!({"type": "resource", "condition": {
                    "resource_type": "memory", "threshold": 101, "operator": "less"
                }})],
                vec![json!({"command": format!("touch {}", marker("resource"))})],
            ),
        ),
    ];

    let mut daemon = Daemon::start(&jobs);
    let startup = daemon.jobs();
    assert_eq!(startup.len(), jobs.len());
    assert!(startup.iter().all(|job| job["status"] == "Unknown"));

    assert_eq!(daemon.post("/start", None).0, 200);
    assert_eq!(daemon.post("/start", None).0, 409);

    daemon.wait_for_jobs(&[
        ("job_cmd", "Completed"),
        ("job_cmd_output", "Completed"),
        ("job_cmd_fail", "Unsatisfied"),
        ("job_variable", "Completed"),
        ("job_process", "Completed"),
        ("job_process_absent", "Unsatisfied"),
        ("job_process_not_running", "Completed"),
        ("job_disk", "Completed"),
        ("job_disk_fail", "Unsatisfied"),
        ("job_resource", "Completed"),
    ]);

    for name in [
        "cmd",
        "cmd_output",
        "variable",
        "process",
        "process_not_running",
        "disk",
        "resource",
    ] {
        assert!(dir.join(name).exists(), "task did not run: {name}");
    }
    for name in ["cmd_fail", "process_absent", "disk_fail"] {
        assert!(
            !dir.join(name).exists(),
            "task ran despite failing condition: {name}"
        );
    }

    assert_eq!(daemon.post("/stop", None).0, 200);
    assert_eq!(daemon.post("/stop", None).0, 409);

    daemon.shutdown();
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn cron_ticks_and_failing_tasks_are_logged() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();
    let ticks = dir.join("ticks").display().to_string();

    let jobs = vec![
        (
            "cron.jsonc",
            json!({
                "id": "job_cron",
                "name": "cron",
                "when": {"type": "cron", "trigger": "*/1 * * * * *"},
                "conditions": [],
                "tasks": [{"command": format!("date +%s >> {ticks}")}]
            }),
        ),
        (
            "fail.jsonc",
            job("job_fail", vec![], vec![json!({"command": "exit 7"})]),
        ),
    ];

    let mut daemon = Daemon::start(&jobs);
    assert_eq!(daemon.post("/start", None).0, 200);

    let deadline = Instant::now() + Duration::from_secs(6);
    while Instant::now() < deadline {
        let count = fs::read_to_string(&ticks)
            .map(|text| text.lines().count())
            .unwrap_or(0);
        if count >= 2 {
            break;
        }
        sleep(Duration::from_millis(50));
    }
    let count = fs::read_to_string(&ticks)
        .map(|text| text.lines().count())
        .unwrap_or(0);
    assert!(count >= 2, "cron only ticked {count} times");

    daemon.wait_for_jobs(&[("job_fail", "Completed")]);

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && !daemon.logs().contains("exit status: 7") {
        sleep(Duration::from_millis(50));
    }
    assert!(
        daemon.logs().contains("exit status: 7"),
        "failing task error was not logged"
    );

    daemon.post("/stop", None);
    daemon.shutdown();
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn api_crud_reload_and_status_work() {
    let mut daemon = Daemon::start(&[(
        "existing.jsonc",
        job("job_existing", vec![command_condition("true")], vec![]),
    )]);

    let (status, body) = daemon.get("/status");
    assert_eq!(status, 200);
    assert_eq!(body["running"], false);
    assert_eq!(body["job_count"], 1);

    assert_eq!(daemon.get("/jobs/missing").0, 404);
    assert!(matches!(daemon.post("/jobs", Some(json!({}))).0, 400..=499));

    let created = json!({
        "id": "ignored",
        "name": "created",
        "description": "created by test",
        "conditions": [],
        "tasks": []
    });
    let (status, body) = daemon.post("/jobs", Some(created));
    assert_eq!(status, 200, "create failed: {body:#?}");
    let path = PathBuf::from(body["path"].as_str().expect("created path"));
    assert!(path.exists());
    assert_eq!(path.extension().unwrap(), "jsonc");

    let created_body: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let created_id = created_body["id"].as_str().unwrap().to_string();
    assert!(created_id.starts_with("job_"));
    assert_eq!(path.file_stem().unwrap(), created_id.as_str());
    assert!(daemon.jobs().iter().any(|job| job["id"] == created_id));

    assert_eq!(daemon.get(&format!("/jobs/{created_id}")).0, 200);

    assert_eq!(daemon.delete(&format!("/jobs/{created_id}")).0, 200);
    assert!(!path.exists());
    assert_eq!(daemon.delete("/jobs/job_missing").0, 404);

    let update = json!({
        "id": "job_existing",
        "name": "updated name",
        "description": "updated",
        "conditions": [],
        "tasks": []
    });
    assert_eq!(daemon.put("/jobs/job_missing", update.clone()).0, 404);
    assert_eq!(daemon.put("/jobs/job_existing", update).0, 200);
    assert!(daemon.dir.join("jobs/existing.jsonc").exists());

    let (status, reload) = daemon.post("/reload", None);
    assert_eq!(status, 200);
    assert_eq!(reload["success"], true);
    let jobs = daemon.jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["id"], "job_existing");
    assert_eq!(jobs[0]["name"], "updated name");

    daemon.shutdown();
}

#[test]
fn invalid_jobs_are_skipped() {
    let daemon = Daemon::start(&[
        ("good.jsonc", job("job_good", vec![], vec![])),
        ("bad.jsonc", Value::Null),
    ]);
    let jobs = daemon.jobs();

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["id"], "job_good");
    assert_eq!(jobs[0]["status"], "Unknown");
}

#[test]
fn cli_status_and_stop_work() {
    let mut daemon = Daemon::start(&[]);
    let binary = env!("CARGO_BIN_EXE_autopilot-rs");
    let port = daemon.port.to_string();

    let status = Command::new(binary)
        .arg("status")
        .env("AUTOPILOT_PORT", &port)
        .output()
        .expect("run status");
    assert!(status.status.success());
    assert!(String::from_utf8_lossy(&status.stdout).contains("is running"));

    let stop = Command::new(binary)
        .arg("stop")
        .env("AUTOPILOT_PORT", &port)
        .output()
        .expect("run stop");
    assert!(stop.status.success());

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(Some(_)) = daemon.child.try_wait() {
            break;
        }
        sleep(Duration::from_millis(25));
    }
    assert!(
        daemon.child.try_wait().unwrap().is_some(),
        "daemon did not exit"
    );

    let status = Command::new(binary)
        .arg("status")
        .env("AUTOPILOT_PORT", &port)
        .output()
        .expect("run status");
    assert!(String::from_utf8_lossy(&status.stdout).contains("is not running"));
}
