use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, 
};
use ifconfig_neon_toys::ThreadPool;

fn main() {
    let tcp_listener = TcpListener::bind("[::]:8080").unwrap();
    let listener = tcp_listener;
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

    let mut buf_reader = BufReader::new(&mut stream);

    // Variables to store the specified headers and other information
    let mut method = String::new();
    let mut user_agent = String::new();
    let mut language = String::new();
    let mut referer = String::new();
    let mut encoding = String::new();
    let mut mime_type = String::new();
    let mut charset = String::new();
    let mut x_forwarded_for = String::new();
    let mut keep_alive = String::new();

    // Initial processing to get the method from the request line
    {
        let mut request_line = String::new();
        buf_reader.read_line(&mut request_line).unwrap();
        method = request_line.split_whitespace().next().unwrap_or("").to_string();
    }

    loop {
        let mut line = String::new();
        let bytes_read = buf_reader.read_line(&mut line).unwrap();
        if bytes_read == 0 || line == "\r\n" {
            break;  // End of headers or connection closed
        }

        let parts: Vec<&str> = line.trim().splitn(2, ':').collect();
        if parts.len() == 2 {
            let header_name = parts[0].to_lowercase();
            let header_value = parts[1].trim();
            match header_name.as_str() {
                "user-agent" => user_agent = header_value.to_string(),
                "accept-language" => language = header_value.to_string(),
                "referer" => referer = header_value.to_string(),
                "accept-encoding" => encoding = header_value.to_string(),
                "accept" => mime_type = header_value.to_string(),
                "accept-charset" => charset = header_value.to_string(),
                "x-forwarded-for" => x_forwarded_for = header_value.to_string(),
                "connection" => keep_alive = header_value.to_string(),
                _ => {},
            }
        }
    }

    let port = peer_addr.port();

    let ip_address = if !x_forwarded_for.is_empty() {
        x_forwarded_for.split(',').next().unwrap_or("").trim().to_string()
    } else {
        peer_addr.ip().to_string()
    };
    
    headers_html.push_str(&format!(
        "<tr><td>IP Address</td><td><strong>{}</strong></td></tr>\
        <tr><td>Port</td><td>{}</td></tr>\
        <tr><td>Method</td><td>{}</td></tr>\
        <tr><td>User Agent</td><td>{}</td></tr>\
        <tr><td>Language</td><td>{}</td></tr>\
        <tr><td>Referer</td><td>{}</td></tr>\
        <tr><td>Encoding</td><td>{}</td></tr>\
        <tr><td>MIME Type</td><td>{}</td></tr>\
        <tr><td>Charset</td><td>{}</td></tr>\
        <tr><td>X-Forwarded-For</td><td>{}</td></tr>\
        <tr><td>Keep Alive</td><td>{}</td></tr>",
        ip_address,
        port,
        method,
        user_agent,
        language,
        referer,
        encoding,
        mime_type,
        charset,
        x_forwarded_for,
        keep_alive,
    ));

    let status_line = "HTTP/1.1 200 OK";
    let filename = "response.html";

    let mut contents = std::fs::read_to_string(filename).unwrap();
    contents = contents.replace("{headers_table}", &headers_html);

    let length = contents.len();
    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}",
        status_line = status_line,
        length = length,
        contents = contents
    );

    stream.write_all(response.as_bytes()).unwrap();
}
