#!/bin/bash
cargo run --bin cli 127.0.0.1:8082 set y 2
cargo run --bin cli 127.0.0.1:8082 get y
cargo run --bin cli 127.0.0.1:8080 set y 1
cargo run --bin cli 127.0.0.1:8080 get y
cargo run --bin cli 127.0.0.1:8082 get y
cargo run --bin cli 127.0.0.1:8083 get y
cargo run --bin cli 127.0.0.1:8084 get y
