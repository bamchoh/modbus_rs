use std::sync::mpsc;
use std::thread::JoinHandle;
use std::{thread, time};
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::convert::TryFrom;
use bytes::{Buf, BytesMut, BufMut};

use super::core;
use super::core::ModbusResponseSerde;

const RX_BUF_SIZE: usize = 8096;
const TX_BUF_SIZE: usize = 8096;

pub fn start_client(host: &'static str, port: usize) -> JoinHandle<()> {
    thread::spawn(move || {
        let tcp_write_stream = TcpStream::connect_timeout(
            &format!("{}:{}", host, port).parse().unwrap(),
            Duration::from_secs(1),
        ).expect("Could not connect.");

        let tcp_read_stream = tcp_write_stream.try_clone().unwrap();

        let mut client = ModbusClient::new(255, tcp_read_stream, tcp_write_stream);

        loop {
            let values = client.read_holding_register(0xFFFF, 2);
            println!("{:?}", values);
        }
    })
}

struct ModbusClient<'a> {
    trans_id: u16,
    unit_id: u8,
    reader: Box<dyn Read + 'a>,
    writer: Box<dyn Write + 'a>,
}

impl<'a> ModbusClient<'a> {
    fn new(unit_id: u8, mut read_stream: impl Read + 'a, write_stream: impl Write + 'a) -> ModbusClient<'a> {
        ModbusClient {
            trans_id: 0,
            unit_id: unit_id,
            reader: Box::new(read_stream),
            writer: Box::new(write_stream),
        }
    }

    fn read(&mut self, rx_buf: &mut [u8]) -> usize {
        self.reader.read(rx_buf).expect("read failed")
    }

    fn write(&mut self, write_buf: BytesMut) {
        self.writer.write(&write_buf).expect("write failed");
    }

    fn encode(&mut self, request: ModbusRequest) -> BytesMut {
        let request_buf = match request {
            ModbusRequest::ReadHoldingRegistersRequest(n) => n.encode(),
            ModbusRequest::None => unimplemented!()
        };

        let header = core::ModbusTCPHeader {
            trans_id: self.trans_id,
            proto_id: 0,
            length: 0,
            unit_id: self.unit_id,
        };

        self.trans_id += 1;

        let mut write_buf = BytesMut::with_capacity(1024);

        header.encode(&mut write_buf, request_buf);

        write_buf
    }

    fn decode(&self, rx_buf: &[u8]) -> ModbusResponseSerde {
        let (_, func_code, p) = core::ModbusTCPHeader::decode(rx_buf);
        println!("{0:X}", func_code);
        if func_code == 3 {
            let decoded = core::ReadHoldingRegisterResponse::decode(p)
            Box::new(decoded)
        } else {
            core::ErrorResponse::decode(p)
        }
    }

    fn read_holding_register(&mut self, address: u16, quantity: u16) -> Option<Vec<u16>> {
        let request = core::ReadHoldingRegistersRequest::new(address,quantity);
        let write_buf = self.encode(request);
        self.write(write_buf);
        let mut rx_buf = [0; RX_BUF_SIZE];
        self.read(&mut rx_buf);
        let response = self.decode(&rx_buf);
        if let ModbwlusResponse::ReadHoldingRegisterResponse(n) = response {
            Some(n.values)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Buf;
    use bytes::BufMut;
    use bytes::Bytes;

    #[test]
    fn read_holding_register1() {
        let read_buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xFF, 0x03, 0x02, 0x12, 0x34].reader();
        let write_buf = vec![].writer();
        let mut client = ModbusClient::new(255, read_buf, write_buf);
        let data = client.read_holding_register(0x0000, 0x0001);
        assert_eq!(data.unwrap(), vec![0x1234]);
    }

    #[test]
    fn read_holding_register2() {
        let read_buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xFF, 0x03, 0x04, 0x12, 0x34, 0x56, 0x78].reader();
        let write_buf = vec![].writer();
        let mut client = ModbusClient::new(255, read_buf, write_buf);
        let data = client.read_holding_register(0x0000, 0x0002);
        assert_eq!(data.unwrap(), vec![0x1234, 0x5678]);
    }
}