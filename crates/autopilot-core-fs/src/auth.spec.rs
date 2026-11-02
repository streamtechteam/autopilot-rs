use super::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "autopilot-auth-{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn generated_tokens_are_256_bit_hex() {
    let token = generate_token().unwrap();

    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|character| character.is_ascii_hexdigit()));
    assert_ne!(token, generate_token().unwrap());
}

#[test]
fn token_files_round_trip_and_are_private_on_unix() {
    let dir = temp_dir("round-trip");
    let path = dir.join(API_TOKEN_FILE);

    write_token_file(&path, "abc123").unwrap();

    assert_eq!(read_file_token(&path).unwrap().as_deref(), Some("abc123"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn environment_token_wins_without_a_token_file() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { env::set_var(API_TOKEN_ENV, "from-environment") };

    assert_eq!(
        read_api_token().unwrap().as_deref(),
        Some("from-environment")
    );

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { env::remove_var(API_TOKEN_ENV) };
}

#[test]
fn constant_time_comparison_requires_an_exact_match() {
    assert!(constant_time_eq("secret", "secret"));
    assert!(!constant_time_eq("secret", "secreT"));
    assert!(!constant_time_eq("secret", "secret "));
    assert!(!constant_time_eq("", "secret"));
}
