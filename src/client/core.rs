use std::sync::mpsc;
use std::thread::JoinHandle;
use std::{thread, time};
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::convert::TryFrom;
use bytes::{Buf, BytesMut, BufMut};

use super::super::modbus::core;

pub fn start_client(host: &'static str, port: usize) -> JoinHandle<()> {
    thread::spawn(move || {
        let tcp_stream_write = TcpStream::connect_timeout(
            &format!("{}:{}", host, port).parse().unwrap(),
            Duration::from_secs(1),
        ).expect("Could not connect.");

        let tcp_stream_read = tcp_stream_write.try_clone().unwrap();

        let (tx, rx): (std::sync::mpsc::Sender<i32>, std::sync::mpsc::Receiver<i32>) = mpsc::channel();

        for handle in vec![core::send(tcp_stream_write, rx), core::read(tcp_stream_read,tx)] {
            handle.join().unwrap();
        }
    })
}
