use super::super::modbus::core;

use std::sync::mpsc;
use std::thread::JoinHandle;
use std::{thread, time};
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::convert::TryFrom;
use bytes::{Buf, BytesMut, BufMut};

pub fn start_server(host: &'static str, port: usize) -> JoinHandle<()> {
    let handle = thread::spawn(move || {
        let listener = TcpListener::bind(&format!("{}:{}", host, port)).expect("bind failed");

        for stream in listener.incoming() {
            handle_server(&mut stream.expect("stream failed"));
        }
    });

    thread::sleep(time::Duration::from_millis(1));

    handle
}

fn handle_server(stream: &mut TcpStream) {
    let mut values = vec![12345,12346];

    loop {
        let mut buf = [0 as u8; 64];
        let n = stream.read(&mut buf).expect("read failed");
        let (mod_header, func_code, p) = core::ModbusTCPHeader::decode(&buf);
        println!("{:?}", mod_header);
        let request = match func_code {
            3 => {
                core::ModbusRequest::new_read_holding_register_request(p)
            }
            _ => core::ModbusRequest::None,
        };
        println!("{:?}", request);
        for value in &mut values {
            *value += 1;
        }
        let mut new_values: Vec<u16> = Vec::with_capacity(values.len());
        for value in &values {
            new_values.push(*value);
        }
        // dump(&buf, n);

        let resp = core::ReadHoldingRegisterResponse::new(new_values);
        let write_buf = core::gen_write_buf(core::ModbusResponse::ReadHoldingRegisterResponse(resp));
        stream.write(&write_buf).expect("write failed");
    }
}