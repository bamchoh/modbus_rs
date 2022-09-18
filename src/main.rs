use modbus::server::{core as server_core};
use modbus::client::{core as client_core};

fn main() -> std::io::Result<()> {
    let server_handle = server_core::start_server("127.0.0.1", 502);

    let client_handle = client_core::start_client("127.0.0.1", 502);

    for handle in vec![server_handle, client_handle] {
        handle.join().unwrap();
    }
    Ok(())
}