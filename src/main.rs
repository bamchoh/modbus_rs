use modbus::server;
use modbus::client;

fn main() -> std::io::Result<()> {
    let server_handle = server::start_server("127.0.0.1", 503);

    let client_handle = client::start_client("127.0.0.1", 502);

    for handle in vec![server_handle, client_handle] {
        handle.join().unwrap();
    }
    Ok(())
}