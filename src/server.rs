use crate::core::{ModbusResponse, ModbusRequest};

use super::core;

use std::sync::mpsc;
use std::thread::JoinHandle;
use std::{thread, time};
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::convert::TryFrom;
use bytes::{Buf, BytesMut, BufMut};

const RX_BUF_SIZE: usize = 8096;
const TX_BUF_SIZE: usize = 8096;

pub fn start_server(host: &'static str, port: usize) -> JoinHandle<()> {
    let handle = thread::spawn(move || {
        let listener = TcpListener::bind(&format!("{}:{}", host, port)).expect("bind failed");

        for stream in listener.incoming() {
            let tcp_write_stream = stream.expect("stream failed");
            let tcp_read_stream = tcp_write_stream.try_clone().unwrap();
            communicate(&tcp_read_stream, &tcp_write_stream);
        }
    });

    thread::sleep(time::Duration::from_millis(1));

    handle
}

fn communicate(read_stream: &TcpStream, write_stream: &TcpStream) {
    let mut values = vec![12345,12346];

    loop {
        let request = read(read_stream);

        let response = operate(&mut values);

        write(write_stream, response);
    }
}

fn read(mut read_stream: &TcpStream) -> ModbusRequest {
    let mut rx_buf = [0 as u8; RX_BUF_SIZE];
    let n = read_stream.read(&mut rx_buf).expect("read failed");
    let (_, func_code, p) = core::ModbusTCPHeader::decode(&rx_buf);
    match func_code {
        3 => {
            core::ModbusRequest::new_read_holding_register_request(p)
        }
        _ => core::ModbusRequest::None,
    }
}

fn operate(values: &mut Vec<u16>) -> ModbusResponse {
    let len = values.len();

    let mut new_values: Vec<u16> = Vec::with_capacity(len);

    for value in values {
        *value += 1;
        new_values.push(*value);
    }

    core::ReadHoldingRegisterResponse::new(new_values)
}

fn write(mut write_stream: &TcpStream, resp: ModbusResponse) {
    let write_buf = core::gen_write_buf(resp);
    write_stream.write(&write_buf).expect("write failed");
}