#!/bin/bash
cargo run --bin cli 127.0.0.1:8080 set a 1
cargo run --bin cli 127.0.0.1:8080 set b 2
cargo run --bin cli 127.0.0.1:8080 set c 3
cargo run --bin cli 127.0.0.1:8080 set a 4
