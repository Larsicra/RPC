## rpc
`log.txt`为恢复机制所需文件，格式为设置指令  
```
set a 1
set b 2
set c fin
```
`setting.txt`为配置文件，格式如下  
```
server 127.0.0.1:8080 master 127.0.0.1:8080     // 8080 主节点 8080
server 127.0.0.1:8082 slaveof 127.0.0.1:8080    // 8082 从属于 8080
server 127.0.0.1:8083 slaveof 127.0.0.1:8080
server 127.0.0.1:8084 slaveof 127.0.0.1:8080
proxy 127.0.0.1:8081                            // proxy
```

`.sh`为测试脚本，测试流程如下  
## 脚本测试
由于不太熟悉脚本所以还是只能分批实现：  
#### AOF
（如果模拟初始，请清空log.txt）  
终端1输入`cargo run --bin server 127.0.0.1:8080 master 127.0.0.1:8080`运行服务器（方便起见为主节点）  
终端2运行`set.sh`执行设置  
关闭终端1的服务器后重新开启  
终端2运行`get.sh`查询恢复结果  

#### 主从 / proxy
终端1运行`servers.sh`按照配置文件运行实例  
等待全部启动后，终端2运行`ms.sh`，其中不通过`proxy`直接与从节点的地址连接来测试从节点，测试用例包括从节点`set`报错，以及主节点`set`之后的同步  

终端2运行`proxy.sh`测试`proxy  
`
## 手动测试格式：  
启动服务器：  
```
cargo run --bin server <地址> master <地址（重复）>             // 主节点
cargo run --bin server <地址> slaveof <主节点地址>              // 从节点
cargo run --bin proxy                                          // 读取预设的地址、主从节点信息
```
执行指令：  
```
cargo run --bin cli 127.0.0.1:8080 get a
cargo run --bin cli 127.0.0.1:8080 set a b
cargo run --bin cli 127.0.0.1:8084 ping

```
创建可以执行多条指令的客户端（`quit`指令作为关闭）：  
```
cargo run --bin client          // 指令同前
```