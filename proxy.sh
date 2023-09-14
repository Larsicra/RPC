#!/bin/bash
cargo run --bin cli 127.0.0.1:8081 set y f
cargo run --bin cli 127.0.0.1:8081 get y
cargo run --bin cli 127.0.0.1:8081 get a &
cargo run --bin cli 127.0.0.1:8081 get b
