use std::sync::mpsc;
use std::thread::JoinHandle;
use std::{thread, time};
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::convert::TryFrom;
use bytes::{Buf, BytesMut, BufMut};

pub fn conv_u16(len: usize) -> u16 {
    match u16::try_from(len) {
        Ok(n) => n,
        Err(_) => 0,
    }
}

pub fn gen_write_buf(resp: ModbusResponse) -> BytesMut {
    let resp_buf = match resp {
        ModbusResponse::ReadHoldingRegisterResponse(n) => n.encode(),
        ModbusResponse::None => BytesMut::with_capacity(0),
    };

    let header = ModbusTCPHeader {
        trans_id: 0,
        proto_id: 0,
        length: 0,
        unit_id: 255,
    };

    let mut write_buf = BytesMut::with_capacity(1024);

    header.encode(&mut write_buf, resp_buf);

    write_buf
}

pub fn read(mut tcp_stream: TcpStream, tx: std::sync::mpsc::Sender<i32>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let mut buf = [0 as u8; 64];
            let n = tcp_stream.read(&mut buf).expect("read failed");
            tx.send(0).unwrap();
            dump(&buf, n);
        }
    })
}

fn dump(buf: &[u8], n: usize) {
    for i in 0..n {
        if i != 0 {
            print!(":");
        }
        print!("{0:02X}", buf[i]);
    }
    println!();
}

fn encode_req(req: ModbusRequest) -> BytesMut {
    let resp_buf = match req {
        ModbusRequest::ReadHoldingRegistersRequest(n) => n.encode(),
        ModbusRequest::None => BytesMut::with_capacity(0),
    };

    let header = ModbusTCPHeader {
        trans_id: 0,
        proto_id: 0,
        length: 0,
        unit_id: 255,
    };

    let mut write_buf = BytesMut::with_capacity(1024);

    header.encode(&mut write_buf, resp_buf);

    write_buf
}

pub fn send(mut tcp_stream: TcpStream, rx: std::sync::mpsc::Receiver<i32>) -> JoinHandle<()> {
    thread::spawn(move || loop {
        let req = ReadHoldingRegistersRequest::new(0x1234, 2);
        let write_buf = encode_req(ModbusRequest::ReadHoldingRegistersRequest(req));
        tcp_stream.write(&write_buf).expect("write failed");
        let _ = rx.recv();
    })
}

#[derive(Debug)]
pub struct ModbusTCPHeader {
    pub trans_id: u16,
    pub proto_id: u16,
    pub length: u16,
    pub unit_id: u8,
}

impl ModbusTCPHeader {
    pub fn decode(mut p: &[u8]) -> (ModbusTCPHeader, u8, &[u8]) {
        let trans_id = p.get_u16();
        let proto_id = p.get_u16();
        let length = p.get_u16();
        let unit_id = p.get_u8();
        let header = ModbusTCPHeader {
            trans_id: trans_id,
            proto_id: proto_id, 
            length: length, 
            unit_id: unit_id,
        };
        (header, p.get_u8(), p)
    }

    pub fn encode(&self, write_buf: &mut BytesMut, inner_buf: BytesMut) {
        write_buf.put_u16(self.trans_id);
        write_buf.put_u16(self.proto_id);
        write_buf.put_u16(conv_u16(inner_buf.len()+1));
        write_buf.put_u8(self.unit_id);
        write_buf.put_slice(&inner_buf);
    }
}

#[derive(Debug)]
pub enum ModbusRequest {
    ReadHoldingRegistersRequest(ReadHoldingRegistersRequest),
    None,
}

impl ModbusRequest {
    pub fn new_read_holding_register_request(mut p: &[u8]) -> Self {
        let address = p.get_u16();
        let quantity = p.get_u16();
        let req = ReadHoldingRegistersRequest {
            address: address,
            quantity: quantity,
        };
        ModbusRequest::ReadHoldingRegistersRequest(req)
    }
}

pub enum ModbusResponse {
    ReadHoldingRegisterResponse(ReadHoldingRegisterResponse),
    None,
}

#[derive(Debug)]
pub struct ReadHoldingRegistersRequest {
    address: u16,
    quantity: u16,
}

impl ReadHoldingRegistersRequest {
    pub fn new(address: u16, quantity: u16) -> Self {
        ReadHoldingRegistersRequest { address: address, quantity: quantity }
    }

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(5);
        buf.put_u8(0x03);
        buf.put_u16(self.address);
        buf.put_u16(self.quantity);

        buf
    }
}

#[derive(Debug)]
pub struct ReadHoldingRegisterResponse {
    values: Vec<u16>
}

impl ReadHoldingRegisterResponse {
    pub fn new(values: Vec<u16>) -> Self {
        ReadHoldingRegisterResponse { values: values }
    }
    
    fn byte_len(&self) -> u8 {
        let v = 0x00FF & self.values.len() * 2;
        u8::try_from(v).unwrap()
    }

    pub fn encode(&self) -> BytesMut {
        let mut resp_buf = BytesMut::with_capacity(2 + self.byte_len() as usize);
        resp_buf.put_u8(0x03);
        resp_buf.put_u8(self.byte_len());
        for value in &self.values {
            resp_buf.put_u16(*value);
        }
        resp_buf
    }
}
