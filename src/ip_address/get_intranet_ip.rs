use std::net::UdpSocket;

// copy from https://github.com/egmkang/local_ipaddress/blob/master/src/lib.rs
// Todo: need all ips use https://crates.io/crates/local-ip-address
pub fn get_intranet_ip() -> Option<String> {
  let socket = match UdpSocket::bind("0.0.0.0:0") {
    Ok(s) => s,
    Err(_) => return None,
  };

  match socket.connect("8.8.8.8:80") {
    Ok(()) => (),
    Err(_) => return None,
  };

  match socket.local_addr() {
    Ok(addr) => Some(addr.ip().to_string()),
    Err(_) => None,
  }
}
