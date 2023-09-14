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


    loop {
        let mut req = volo_gen::volo::example::GetItemRequest {
            ops: FastStr::from(""),
            key: FastStr::from(""),
            value: FastStr::from(""),
            port: FastStr::from(addr.clone()),
        };

        println!("");
        println!("input your command");
        let mut buf: String = String::new();
        std::io::stdin().read_line(&mut buf);
        
        let para: Vec<String> = buf.trim().split(" ").map(String::from).collect();

        if para[0] == "set" {
            println!("to set");
            if para.len() != 3 {
                println!("invalid format");
                continue;
            }
            req.ops = FastStr::from("set");
            req.key = FastStr::from(para[1].clone());
            req.value = FastStr::from(para[2].clone());
        } else if para[0] == "get" {
            println!("to get");
            if para.len() != 2 {
                println!("invalid format");
                continue;
            }
            req.ops = FastStr::from("get");
            req.key = FastStr::from(para[1].clone());
        } else if para[0] == "del" {
            println!("to del");
            if para.len() != 2 {
                println!("invalid format");
                continue;
            }
            req.ops = FastStr::from("del");
            req.key = FastStr::from(para[1].clone());
        } else if para[0] == "ping" {
            println!("to ping");
            req.ops = FastStr::from("ping");
            if para.len() > 1 {
                req.key = FastStr::from(para[1].clone());
            }
        } else if para[0] == "quit" {
            req.ops = FastStr::from("quit");
            tracing::info!("quit");
            // break;
        } else {
            tracing::info!("invalid opcode");
            continue;
        }
        
        let resp = client.get_item(req).await;
        match resp {
            Ok(info) => {
                if para[0] == "get" {
                    if info.stat {
                        tracing::info!("value of {} is {}", para[1], info.value.to_string());
                    } else {
                        tracing::info!("no value for {}", para[1]);
                    }
                } else if para[0] == "set" {
                    tracing::info!("set success");
                } else if para[0] == "del" {
                    if info.stat {
                        tracing::info!("{} deleted", para[1]);
                    } else {
                        tracing::info!("no key as {}", para[1]);
                    }
                } else if para[0] == "ping" {
                    println!("{}", info.value.to_string());
                } else if para[0] == "quit" {
                    tracing::info!("bye");

                    break;
                }
                // tracing::info!("{:?}", info)
            }
            Err(e) => {
                tracing::error!("error: {}", e.to_string());
                // tracing::error!("{:?}", e)
            }
        }

    }



    // run the get_item, which is from lib.rs

    // see from status

}
