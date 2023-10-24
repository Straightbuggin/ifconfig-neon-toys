extern crate reqwest;

use ifconfig_neon_toys::ThreadPool;
use reqwest::Error;
use serde::Deserialize;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use tokio::runtime::Runtime;

fn main() {
    let tcp_listener = TcpListener::bind("[::]:8080").unwrap();
    let listener = tcp_listener;
    let pool = ThreadPool::new(100);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| handle_connection(stream));
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

    {
        let mut request_line = String::new();
        buf_reader.read_line(&mut request_line).unwrap();
        method = request_line
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
    }

    loop {
        let mut line = String::new();
        let bytes_read = buf_reader.read_line(&mut line).unwrap();
        if bytes_read == 0 || line == "\r\n" {
            break;
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
                _ => {}
            }
        }
    }

    let port = peer_addr.port();

    let ip_address = if !x_forwarded_for.is_empty() {
        x_forwarded_for
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    } else {
        peer_addr.ip().to_string()
    };

    let ip_info = Runtime::new()
        .unwrap()
        .block_on(fetch_ip_info(&ip_address))
        .unwrap();

    let ip_info_rows = vec![
        ("Continent", ip_info.continent.clone()),
        ("Continent Code", ip_info.continent_code.clone()),
        ("Country", ip_info.country.clone()),
        ("Country Code", ip_info.country_code.clone()),
        ("Region", ip_info.region.clone()),
        ("Region Name", ip_info.region_name.clone()),
        ("City", ip_info.city.clone()),
        ("District", ip_info.district.clone()),
        ("ZIP Code", ip_info.zip.clone()),
        ("Latitude", ip_info.lat.to_string()),
        ("Longitude", ip_info.lon.to_string()),
        ("Timezone", ip_info.timezone.clone()),
        ("Offset", ip_info.offset.to_string()),
        ("Currency", ip_info.currency.clone()),
        ("ISP", ip_info.isp.clone()),
        ("Organization", ip_info.org.clone()),
        ("AS Name", ip_info.asname.clone()),
        ("Reverse DNS", ip_info.reverse.clone()),
        ("Mobile", ip_info.mobile.to_string()),
        ("Proxy", ip_info.proxy.to_string()),
        ("Hosting", ip_info.hosting.to_string()),
    ];

    let header_rows = vec![
        ("IP Address", ip_address.to_string()),
        ("Port", port.to_string()),
        ("Method", method.clone()),
        ("User Agent", user_agent.clone()),
        ("Language", language.clone()),
        ("Referer", referer.clone()),
        ("Encoding", encoding.clone()),
        ("MIME Type", mime_type.clone()),
        ("Charset", charset.clone()),
        ("X-Forwarded-For", x_forwarded_for.clone()),
        ("Keep Alive", keep_alive.clone()),
    ];

    let headers_html = create_table("Your Information", header_rows);
    let ip_info_table = create_table("Your IP Information", ip_info_rows);

    let response_body = format!(
        "<html>\
        <head>\
            <style>\
                h1 {{\
                    text-align: center;\
                    padding-top: 20px;\
                }}\
                body {{\
                    font-family: Arial, sans-serif;\
                    margin: 0;\
                    padding: 0;\
                    background-color: #f0f0f0;\
                }}\
                table {{\
                    border-collapse: collapse;\
                    margin: 20px auto;\
                    background-color: #ffffff;\
                }}\
                th, td {{\
                    border: 1px solid #ddd;\
                    text-align: left;\
                    padding: 8px;\
                }}\
                th {{\
                    background-color: #2c3e50;\
                    color: white;\
                }}\
            </style>\
        </head>\
        <body>\
            <h1>What's my IP?</h1>\
            {}\
            {}\
        </body>\
    </html>",
        headers_html,
        ip_info_table
    );

    let status_line = "HTTP/1.1 200 OK";
    let length = response_body.len();
    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{response_body}",
        status_line = status_line,
        length = length,
        response_body = response_body
    );

    stream.write_all(response.as_bytes()).unwrap();
}

#[derive(Deserialize)]
struct IpInfo {
    continent: String,
    #[serde(rename = "continentCode")]
    continent_code: String,
    country: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    region: String,
    #[serde(rename = "regionName")]
    region_name: String,
    city: String,
    district: String,
    zip: String,
    lat: f64,
    lon: f64,
    timezone: String,
    offset: i64,
    currency: String,
    isp: String,
    org: String,
    asname: String,
    reverse: String,
    mobile: bool,
    proxy: bool,
    hosting: bool,
}

async fn fetch_ip_info(ip: &str) -> Result<IpInfo, Error> {
    let var_name = format!("http://ip-api.com/json/{}?fields=message,continent,continentCode,country,countryCode,region,regionName,city,district,zip,lat,lon,timezone,offset,currency,isp,org,asname,reverse,mobile,proxy,hosting,query", ip);
    let url = var_name;
    let response = reqwest::get(&url).await?;
    let ip_info: IpInfo = response.json().await?;
    Ok(ip_info)
}

fn create_table(title: &str, rows: Vec<(&str, String)>) -> String {
    let mut table_rows = String::new();
    for (key, value) in rows {
        table_rows.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", key, value));
    }

    format!(
        "<table>\
            <thead>\
                <tr>\
                    <th colspan=\"2\">{}</th>\
                </tr>\
            </thead>\
            <tbody>\
                {}\
            </tbody>\
        </table>",
        title, table_rows
    )
}
