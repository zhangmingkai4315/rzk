extern crate failure;
extern crate futures;
extern crate tokio;

use failure::{Error, ResultExt};
use tokio::prelude::*;
use std::net::SocketAddr;
use tokio::net::TcpStream;


pub struct Zookeeper{}

impl Zookeeper{
    fn connect(addr: &SocketAddr) -> impl Future<Item=Self, Error=failure::Error>{
        tokio::net::TcpStream::connect(addr).and_then(|stream|{
            Self::handshake(stream)
        })
    }

    async fn handshake(stream: tokio::net::TcpStream) ->impl Future<Item=Self, Error=failure::Error>{
        let request = proto::Connection{};
        let mut stream = proto::wrap(stream);
        stream.send(request).and_then(|stream|{
            stream.receive()
        }).and_then(|(response, stream)|{
            Zookeeper{}
        })
    }
}

mod proto;


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn connect(){
        let zk = tokio::run(Zookeeper::connect())
    }
}

