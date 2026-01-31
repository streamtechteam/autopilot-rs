build args="":
    cargo build --release {{args}} && mkdir build && cp target/release/auto_pilot_rs ./build/auto_pilot_rs
run_release args="":
    cargo run --release {{args}}
run_debug args="":
    cargo run {{args}}
clean:
    rm -rf build
