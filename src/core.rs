use std::fmt::Error;
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

pub fn gen_write_buf<T: ModbusResponseSerde>(resp: Option<T>) -> BytesMut {
    let resp_buf = match resp {
        Some(n) => n.encode(),
        None => BytesMut::with_capacity(0),
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

pub fn dump(buf: &[u8], n: usize) {
    for i in 0..n {
        if i != 0 {
            print!(":");
        }
        print!("{0:02X}", buf[i]);
    }
    println!();
}

pub fn encode_req(tx_buf: &mut BytesMut, req: ModbusRequest) {
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

    header.encode(tx_buf, resp_buf);
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct ReadHoldingRegistersRequest {
    pub address: u16,
    pub quantity: u16,
}

impl ReadHoldingRegistersRequest {
    pub fn new(address: u16, quantity: u16) -> ModbusRequest {
        ModbusRequest::ReadHoldingRegistersRequest(
            ReadHoldingRegistersRequest { address: address, quantity: quantity }
        )
    }

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(5);
        buf.put_u8(0x03);
        buf.put_u16(self.address);
        buf.put_u16(self.quantity);

        buf
    }
}

pub enum ModbusResponse {
    Some(ReadHoldingRegisterResponse),
    None,
}

pub trait ModbusResponseSerde {
    fn encode(&self) -> BytesMut;
    fn decode(buf: &[u8]) -> Self;
}

#[derive(Debug)]
pub struct ReadHoldingRegisterResponse {
    pub values: Vec<u16>
}

impl ReadHoldingRegisterResponse {
    pub fn new(values: Vec<u16>) -> Self {
        ReadHoldingRegisterResponse { values: values }
    }
    
    fn byte_len(&self) -> u8 {
        let v = 0x00FF & self.values.len() * 2;
        u8::try_from(v).unwrap()
    }

    pub fn decode2(mut buf: &[u8]) -> Self {
        ReadHoldingRegisterResponse::decode(buf)
    }
}

impl ModbusResponseSerde for ReadHoldingRegisterResponse {
    fn encode(&self) -> BytesMut {
        let mut resp_buf = BytesMut::with_capacity(2 + self.byte_len() as usize);
        resp_buf.put_u8(0x03);
        resp_buf.put_u8(self.byte_len());
        for value in &self.values {
            resp_buf.put_u16(*value);
        }
        resp_buf
    }

    fn decode(mut buf: &[u8]) -> Self {
        let n = buf.get_u8() as usize / 2;

        let mut values: Vec<u16> = Vec::with_capacity(n);

        for i in 0..n {
            values.push(buf.get_u16());
        }

        Self::new(values)
    }
}


pub struct ErrorResponse();

impl ModbusResponseSerde for ErrorResponse {
    fn encode(&self) -> BytesMut {
        BytesMut::with_capacity(0)
    }

    fn decode(buf: &[u8]) -> Self {
        ErrorResponse()
    }
}