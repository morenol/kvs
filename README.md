# kvs [![Actions Status](https://github.com/morenol/kvs/workflows/CI/badge.svg)](https://github.com/morenol/kvs/actions)

Key-Value ruStorage

  Implementation of a Key-Value storage is rust following the rust course from [talent plan](https://github.com/pingcap/talent-plan/).
  
 ## Note
 This project was done with the intention to learn rust. Do not use it if you have other intentions.
  
 ## Usage
 
 This project provides two binaries and a library to use the kvs engine.
 
 ### Library
 
 ```rust
 use kvs::KvsStore;
 
 fn main(){
     let store = KvsStore::open(".");
     store.set("foo".to_borrowed(), "bar".to_borrowed()).unwrap();
     
     assert_eq!(store.get("foo".to_borrowed()).unwrap(), Some("bar".to_borrowed());
     store.remove("foo".to_borrowed()).unwrap();
     
     assert_eq!(store.get("foo".to_borrowed()).unwrap(), None);
 }
 ```
 
 ### Binaries
 
 Two binaries are provided, namely kvs-server and kvs-client.
 
**kvs-server**
  
```
USAGE:
    kvs-server [OPTIONS]

OPTIONS:
        --addr <IP-PORT>          Bind server to a given IP address and a port number, with the format IP:PORT [default:
                                  127.0.0.1:4000]
        --engine <ENGINE-NAME>    Sets server engine. Use 'kvs' or 'sled'.
```
**kvs-client**

```
USAGE:
    kvs-client <SUBCOMMAND> [OPTIONS] <PARAMETERS>

OPTIONS:
        --addr <IP-PORT>    Sets IP servers address and a port number, with the format IP:PORT [default: 127.0.0.1:4000]

SUBCOMMANDS:
    get  <KEY>          Gets the value of a given key
    rm   <KEY>          Remove a given key from the KV storage/
    set  <KEY> <VALUE>  Sets a value for a given key.
```

## Documentation:
  
 This will get some love in the future.
  
  
  
  
