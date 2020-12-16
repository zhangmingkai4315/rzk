use std::io;

use byteorder::{BigEndian, WriteBytesExt};
use tokio;
use tokio::prelude::*;

use crate::Zookeeper;
use futures::StartSend;

#[repr(i32)]
#[allow(dead_code)]
pub(super) enum OpCode {
    Notification = 0,
    Create = 1,
    Delete = 2,
    Exists = 3,
    GetData = 4,
    SetData = 5,
    GetACL = 6,
    SetACL = 7,
    GetChildren = 8,
    Synchronize = 9,
    Ping = 11,
    GetChildren2 = 12,
    Check = 13,
    Multi = 14,
    Auth = 100,
    SetWatches = 101,
    Sasl = 102,
    CreateSession = -10,
    CloseSession = -11,
    Error = -1,
}

pub(crate) struct Packetizer<S> {
    stream: S,

    /// bytes we have not yet set
    outbox: Vec<u8>,

    /// prefix of outbox that has been sent
    outstart: usize,

    /// bytes we have not yet deserialized
    inbox: Vec<u8>,

    /// prefix of inbox that has been sent
    instart: usize,
    xid: i32,
}

impl<S> Packetizer<S> {
    pub(crate) fn new(stream: S) -> Packetizer<S> {
        Packetizer {
            stream,
            outbox: vec![],
            outstart: 0,
            inbox: vec![],
            instart: 0,
            xid: 0,
        }
    }
}

pub(crate) enum ZookeeperRequest {
    Connect {
        protocol_version: i32,
        last_zxid_seen: i64,
        timeout: i32,
        session_id: i64,
        passwd: Vec<u8>,
        read_only: bool,
    },
}

impl ZookeeperRequest {
    fn serialize_into(&self, buffer: &mut Vec<u8>) -> Result<usize, io::Error> {
        match self {
            ZookeeperRequest::Connect {
                protocol_version,
                last_zxid_seen,
                timeout,
                session_id,
                passwd,
                read_only,
            } => {
                let start = buffer.len();
                buffer.write_i32::<BigEndian>(*protocol_version);
                buffer.write_i64::<BigEndian>(*last_zxid_seen);
                buffer.write_i32::<BigEndian>(*timeout);
                buffer.write_i64::<BigEndian>(*session_id);
                buffer.write_i32::<BigEndian>(passwd.len() as i32);
                buffer.write_all(&*passwd);
                buffer.write_u8(read_only as u8);
                Ok(buffer.len() - start)
            }
        }
    }
}

impl<S> Sink for Packetizer<S>
where
    S: AsyncWrite,
{
    type SinkItem = ZookeeperRequest;
    type SinkError = failure::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        let lengthi = self.outbox.len();
        self.outbox.push(0);
        self.outbox.push(0);
        self.outbox.push(0);
        self.outbox.push(0);

        if let ZookeeperRequest::Connect { .. } = item {
        } else {
            self.outbox
                .write_i32::<BigEndian>(self.xid)
                .expect("vec:: should never fail");
        };

        let written = self.outbox.len() - lengthi - 4;

        item.serialize_into(&mut self.outbox)
            .expect("write should never fail");

        let length = &mut self.outbox[lengthi..lengthi + 4];
        length
            .write_i32::<BigEndian>(written as i32)
            .expect("vec should never fail");
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        let n = try_ready!(self.stream.write(&self.outbox[self.outstart..]));
        if self.outstart == self.outbox.len() {
            self.outbox.clear();
            self.outstart = 0;
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        unimplemented!()
    }
}
