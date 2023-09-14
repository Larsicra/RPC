#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;

use volo_example::{S, Proxy};

use volo_example::LogLayer;



#[volo::main]
async fn main() {
    let addr: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    let inn = Proxy::new().await;
    let inner = inn.unwrap();

    volo_gen::volo::example::ItemServiceServer::new(inner)
    .layer_front(LogLayer)
    .run(addr)
    .await
    .unwrap();
}
