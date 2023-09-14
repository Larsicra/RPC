#![feature(impl_trait_in_assoc_type)]

#![allow(warnings, unused)]

use std::net::SocketAddr;

use volo_example::{S};

use volo_example::LogLayer;

use std::env;


#[volo::main]
async fn main() {
    // as "cargo run --bin server 127.0.0.1:8081 slaveof 127.0.0.1:8080"
    // as "cargo run --bin server 127.0.0.1:8081 master 127.0.0.1:8081"
    // 4
    // ["target/debug/server", "127.0.0.1:8081", "slaveof", "127.0.0.1:8080"]
    // tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    let addr: SocketAddr = args[1].parse().unwrap();
    let addr = volo::net::Address::from(addr);
    // tracing_subscriber::fmt::init();
    let is_slave = if args[2] == "slaveof" {
        true
    } else {
        false
    };

    let inn = S::new(is_slave, args[1].to_string(), args[3].to_string()).await;
    let inner = inn.unwrap();

    let ns = volo_gen::volo::example::ItemServiceServer::new(inner);
    println!("init server");

    let fin = ns.layer_front(LogLayer)
    .run(addr)
    .await
    .unwrap();
}
