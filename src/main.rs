use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, 
};
use ifconfig_neon_toys::ThreadPool;

fn main() {
    let listener = TcpListener::bind("[::]:8080").unwrap();
    let pool = ThreadPool::new(100);



    
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream)
        });
    }
    println!("shutting down.");
}

fn handle_connection(mut stream: TcpStream) {

    let mut headers_html = String::new();

    let peer_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Could not retrieve peer address: {}", e);
            return;
        }
    };

    let ip_address = peer_addr.ip();

    let formatted_ip = match ip_address {
        std::net::IpAddr::V4(v4_addr) => format!("{}", v4_addr),
        std::net::IpAddr::V6(v6_addr) => match is_ipv4_mapped(v6_addr) {
            Some(v4_addr) => format!("{}", v4_addr),
            None => format!("{}", v6_addr),
        },
    };

    let mut buf_reader = BufReader::new(&mut stream);


    headers_html.push_str(&format!("<tr><td>IP Address</td><td><strong>{}</strong></td></tr>", formatted_ip));
    loop {
        let mut line = String::new();
        let bytes_read = buf_reader.read_line(&mut line).unwrap();
        if bytes_read == 0 || line == "\r\n" {
            break;  // End of headers or connection closed
        }
       
        let parts: Vec<&str> = line.trim().splitn(2, ':').collect();
        if parts.len() == 2 {
            headers_html.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", parts[0], parts[1].trim()));
        }
    }

    

    
    let status_line = "HTTP/1.1 200 OK";
    let filename = "response.html";
    
    let mut contents = fs::read_to_string(filename).unwrap();
    contents = contents.replace("{headers_table}", &headers_html); 
    
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}", status_line = status_line, length = length, contents = contents);
    
    stream.write_all(response.as_bytes()).unwrap();
}

fn is_ipv4_mapped(ipv6: std::net::Ipv6Addr) -> Option<std::net::Ipv4Addr> {
    let octets = ipv6.octets();
    if &octets[0..10] == [0; 10] && octets[10] == 0xff && octets[11] == 0xff {
        Some(std::net::Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15]))
    } else {
        None
    }
}
