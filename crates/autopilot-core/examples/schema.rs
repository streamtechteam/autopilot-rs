use autopilot_core::job::JobScheme;

fn main() {
    let schema = schemars::schema_for!(JobScheme);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
