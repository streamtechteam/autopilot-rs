daemon:
    cd daemon && cargo build --release --package auto_pilot_rs-daemon && cp target/release/auto_pilot_rs-daemon ../build
cli:
    cd cli && cargo build --release --package auto_pilot_rs-cli && cp target/release/auto_pilot_rs-cli ../build


all: daemon cli
