#![allow(warnings, unused)]

use lazy_static::lazy_static;
use pilota::FastStr;
use std::net::SocketAddr;

use volo_example::LogLayer;

use std::env;


lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
}

#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    let addr = args[1].clone();

    let client: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = addr.parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };


    let mut req = volo_gen::volo::example::GetItemRequest {
        ops: FastStr::from(""),
        key: FastStr::from(""),
        value: FastStr::from(""),
        port: FastStr::from(addr.clone()),
    };

    if args[2] == "set" {
        println!("to set");
        if args.len() != 5 {
            println!("invalid format");
        }
        req.ops = FastStr::from("set");
        req.key = FastStr::from(args[3].clone());
        req.value = FastStr::from(args[4].clone());
    } else if args[2] == "get" {
        println!("to get");
        if args.len() != 4 {
            println!("invalid format");
        }
        req.ops = FastStr::from("get");
        req.key = FastStr::from(args[3].clone());
    } else if args[2] == "del" {
        println!("to del");
        if args.len() != 4 {
            println!("invalid format");
        }
        req.ops = FastStr::from("del");
        req.key = FastStr::from(args[3].clone());
    } else if args[2] == "ping" {
        println!("to ping");
        req.ops = FastStr::from("ping");
        if args.len() > 3 {
            req.key = FastStr::from(args[3].clone());
        }
    } else {
        tracing::info!("invalid opcode");
    }
    
    let resp = client.get_item(req).await;
    match resp {
        Ok(info) => {
            if args[2] == "get" {
                if info.stat {
                    tracing::info!("value of {} is {}", args[3], info.value.to_string());
                } else {
                    tracing::info!("no value for {}", args[3]);
                }
            } else if args[2] == "set" {
                tracing::info!("set success");
            } else if args[2] == "del" {
                if info.stat {
                    tracing::info!("{} deleted", args[3]);
                } else {
                    tracing::info!("no key as {}", args[3]);
                }
            } else if args[2] == "ping" {
                println!("{}", info.value.to_string());
            }
            // tracing::info!("{:?}", info)
        }
        Err(e) => {
            tracing::error!("error: {}", e.to_string());
            // tracing::error!("{:?}", e)
        }
    }

    
}
