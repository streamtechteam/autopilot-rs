build +args="":
    mkdir build
    cargo build --release {{ args }} && cp target/release/autopilot-rs ./build/autopilot_rs

run_release +args="":
    cargo run --release {{ args }}

run_debug +args="":
    cargo run {{ args }}

clean:
    rm -rf build
