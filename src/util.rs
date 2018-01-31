
use std::io;
use std::net::TcpListener;

/// Find a TCP port number to use. This is racy but see
/// https://bugzilla.mozilla.org/show_bug.cgi?id=1240830
///
/// If port is Some, check if we can bind to the given port. Otherwise
/// pick a random port.
pub fn check_tcp_port(port: Option<u16>) -> io::Result<u16> {
    TcpListener::bind(&("localhost", port.unwrap_or(0)))
        .and_then(|stream| stream.local_addr())
        .map(|x| x.port())
}

