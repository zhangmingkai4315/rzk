use tokio;
use tokio::prelude::*;

use crate::Zookeeper;
use futures::StartSend;

struct Packetizer<S>{
    stream: S,
    outbox: Vec<u8>,
    outstart: usize,
    inbox: Vec<u8>,
    instart: usize,
}

fn wrap<S>(stream: S)->Packetizer<S>
    where S: AsyncRead + AsyncWrite
{
    Packetizer{ stream, outbox: vec![], outstart: 0, inbox: vec![], instart: 0 }
}

enum ZookeeperRequest{
    ConnectRequest{
        protocol_version: i32,
        last_zxid_seen: i64,
        timeout: i32,
        session_id: i64,
        passwd: Vec<u8>,
        read_only: bool,
    }
}



impl <S> Sink for Packetizer<S> where S: AsyncWrite{
    type SinkItem = ZookeeperRequest;
    type SinkError = failure::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        let type_and_payload = item.serialize();
        let xid = self.xid;
        let length = type_and_payload.length() + 4;
        self.outbox.push(length);
        self.outbox.push(xid);
        self.outbox.append(type_and_payload);
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        let n = try_ready!(self.stream.write(&self.outbox[self.outstart..]));
        if self.outstart == self.outbox.len(){
            self.outbox.clear();
            self.outstart = 0;
            Ok(Async::Ready(()))
        }else{
            Ok(Async::NotReady)
        }
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        unimplemented!()
    }
}