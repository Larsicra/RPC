#![feature(impl_trait_in_assoc_type)]

#![allow(warnings, unused)]

use std::collections::HashMap;
use tokio::sync::Mutex;

use anyhow::{Error, anyhow};
use pilota::FastStr;

use std::fs::File;
use std::io::{Write, BufReader, BufRead};

use std::fs::OpenOptions;

use std::net::SocketAddr;

use std::thread;
use std::time::Duration;

use futures::{select, FutureExt, pin_mut};

// mod proxy;

pub struct S {
    data: Mutex<HashMap<String, String>>,
    connx: Mutex<Vec<String>>,      // if slave, the only value is to the master
    is_slave: bool,
    sel_addr: String,
}

impl S {
    pub async fn new(is_slave: bool, addr: String, master_addr: String) -> Result<Self, Error> {

        let mut hsmap = HashMap::new();
        let conn: Vec<String> = Vec::new();

        let path = "log.txt";                                           // read from log                                     
        let input = File::open(path)?;
        let buffered = BufReader::new(input);
        for line in buffered.lines() {
            let st = line?;
            let line_str: Vec<String> = st.split(" ").map(String::from).collect();
            if line_str.len() != 3 {
                return Err(anyhow!("invalid log read").into());   
            }

            if line_str[0] == "set" {
                hsmap.insert(line_str[1].clone(), line_str[2].clone());
            }
        }

        let n = Self {
            data: Mutex::new(hsmap),
            connx: Mutex::new(conn),
            is_slave: is_slave,
            sel_addr: addr,
        };

        if is_slave {
            n.connect_to(master_addr).await;
        }

        Ok(n)
        
    }

    async fn connect_to(&self, target: String) {                            // connect to master
        let n_client: volo_gen::volo::example::ItemServiceClient = {
            let addr: SocketAddr = target.parse().unwrap();
            volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
                .layer_outer(LogLayer)
                .address(addr)
                .build()
        };

        let req = volo_gen::volo::example::GetItemRequest {
            ops: FastStr::from("slave"),
            key: FastStr::from(FastStr::from(self.sel_addr.clone())),
            value: FastStr::from(""),
            port: FastStr::from(""),
        };

        let _ = n_client.get_item(req).await;
        self.connx.lock().await.push(target);
        
        println!("became slave");
    }

}

async fn els_to_slave(target: String, key: String, val: String, port: String) {
    let n_client: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = target.parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
    let req = volo_gen::volo::example::GetItemRequest {
        ops: FastStr::from("set"),
        key: FastStr::from(key.clone()),
        value: FastStr::from(val.clone()),
        port: FastStr::from(port.clone()),
    };
    n_client.get_item(req).await;
}


async fn write_log(key: String, val: String) {
    let mut output = OpenOptions::new().append(true).open("log.txt").expect("cannot open file");
    writeln!(output, "set {} {}", key, val);
}

#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
	async fn get_item(&self, _req: volo_gen::volo::example::GetItemRequest)
     -> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError> 
    {
        thread::sleep(Duration::from_secs(5));

        let mut res = volo_gen::volo::example::GetItemResponse {
            value: FastStr::from(""),
            stat: false,
        };

        if _req.ops == "set".to_string() {
            let set_from = _req.port.into_string().clone();

            if self.is_slave {
                let targets = self.connx.lock().await;
                for target in targets.iter() {
                    if !set_from.eq(target) {
                        return Err(Error::msg("can't set from slave"));
                    }
                }
            }
            let k = _req.key.into_string().clone();
            let v = _req.value.into_string().clone();
            self.data.lock().await.insert(k.clone(), v.clone());                        // insert the data

            res.stat = true;                                                            // finish

            if !self.is_slave {                                                        
                write_log(k.clone(), v.clone()).await;                              // only the master write to the log

                let targets = self.connx.lock().await;

                let mut s = Vec::new();

                for target in targets.iter() {
                    println!("to send to slave {}", target);

                    s.push(els_to_slave(target.clone(), k.clone(), v.clone(), self.sel_addr.clone()));
                }

                futures::future::join_all(s).await;
                println!("send finished");
            }
        } else if _req.ops == "get".to_string() {
            let k = _req.key.to_string();
            match self.data.lock().await.get(&k) {
                Some(get_res) => {
                    res.stat = true;
                    res.value = FastStr::from(get_res.clone());
                }
                None => {
                    res.stat = false;
                }
            }
        } else if _req.ops == "del".to_string() {
            let k = _req.key.to_string();
            match self.data.lock().await.remove(&k) {
                Some(_) => {
                    res.stat = true;
                } 
                None => {
                    res.stat = false;
                }
            }

        } else if _req.ops == "ping".to_string() {
            if _req.key.len() == 0 {
                res.value = FastStr::from("PONG");
            } else {
                res.value = FastStr::from(_req.key.clone());
            }
            res.stat = true;
        } else if _req.ops == "slave".to_string() {  
            println!("to get slave at {}", _req.key.clone().into_string());
            let target = _req.key.to_string().clone();
            self.connx.lock().await.push(_req.key.into_string());

            for (k, v) in self.data.lock().await.iter() {                                                               // async 
                tokio::spawn(els_to_slave(target.clone(), k.to_string(), v.to_string(), self.sel_addr.clone())).await;
            }
            // send all to the new slave:
        } else if _req.ops == "quit".to_string() {
            
        } else {
                return Err(Error::msg("invalid opcode"));
        }

        Ok(res)
    }
}


//////////////////////////////////////////////////// LogLayer

#[derive(Clone)]
pub struct LogService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    Cx: Send + 'static,
    anyhow::Error: Into<S::Error>,
{
    // type Req = volo_gen::volo::example::GetItemRequest;

    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let illegals = vec!["fword", "bword"];                              // filter example: fword / nword
        let r = format!("{:?}", &req);
        
        for ill in illegals {
            if r.contains(ill) {
                return Err(anyhow!("illegal words detected").into());                                   // contains the words to be filtered
            }
        }


        let now = std::time::Instant::now();
        let resp = self.0.call(cx, req).await;
        tracing::info!("Request took {}ms", now.elapsed().as_millis());
        resp
    }
}

pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}

/////////////////////////////////////////////////////////////////////////////

pub struct Proxy {
    master: String,
    connx: Mutex<Vec<String>>,
}

impl Proxy {
    pub async fn new() -> Result<Self, Error> {
        let mut conn: Vec<String> = Vec::new();
        let mut master = String::new();
        // let mut self_port = String::new();

        let path = "setting.txt";                                           // read from settings                              
        let input = File::open(path)?;
        let buffered = BufReader::new(input);
        for line in buffered.lines() {
            let st = line?;
            let line_str: Vec<String> = st.split(" ").map(String::from).collect();
            
            if line_str[0] == "proxy" {

            } else if line_str[2] == "master" {
                println!("pro master: {}", line_str[2].clone());
                master = line_str[1].clone();
            } else if  line_str[2] == "slaveof" {
                println!("pro slave: {}", line_str[2].clone());
                conn.push(line_str[1].clone());
            }
        }

        let n = Proxy {
            master: master,
            connx: Mutex::new(conn),
        };
        Ok(n)
    }


}

async fn to_set(target: String, _req: volo_gen::volo::example::GetItemRequest)
-> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError> {
    let n_client: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = target.parse().unwrap();
        println!("{:?}", addr.clone());
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
    let res = n_client.get_item(_req).await.unwrap();
   Ok(res)
}

async fn to_slave(target: String, _req: volo_gen::volo::example::GetItemRequest)
-> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError> {
    let n_client: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = target.parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };

    let res = n_client.get_item(_req).await.unwrap();

    Ok(res)
}

#[volo::async_trait]
impl volo_gen::volo::example::ItemService for Proxy {
	async fn get_item(&self, _req: volo_gen::volo::example::GetItemRequest)
     -> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError> 
    {

        if _req.ops == "set".to_string() {
            println!("to set");
            let handle = tokio::spawn(to_set(self.master.clone(), _req));
            let res = handle.await.unwrap();
            return res;
        } else {
            println!("to get");
            let target = self.connx.lock().await;

            let h1 = to_slave(target[0].clone(), _req.clone()).fuse();
            let h2 = to_slave(target[1].clone(), _req.clone()).fuse();
            let h3 = to_slave(target[2].clone(), _req.clone()).fuse();

            pin_mut!(h1, h2, h3);

            let item = select! {
                res = h1 => res,
                res = h2 => res,
                res = h3 => res,
            };
            return item;
            // let res = tokio::spawn(send_to(tar, _req, self.self_port.clone())).await.unwrap();
            // // let res = send_to(tar, _req, self.self_port.clone());

            // return res;
        }
    }
}

