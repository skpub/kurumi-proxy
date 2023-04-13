use clap::{App, Arg};
use reqwest::header;
// use std::io::{self, Read, Write};
use std::str;
use std::io::Write;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;

// fn copy(reader: &mut dyn Read, writer: &mut dyn Write, output: &str, first: bool) {
//     const BUFFER_SIZE: usize = 32 * 1024;
//     let mut buf = [0u8; BUFFER_SIZE];

//     if first {
//         let mut header_str = format!("POST / HTTP/1.1\n");
//         header_str += format!("Host: {}\n", output).as_str();
//         header_str += format!("User-Agent: kurumi-proxy\n").as_str();
//         header_str += format!("Accept: */*\n").as_str();
//         header_str += format!("Content-Type: application/x-www-form-urlencoded\n").as_str();

//         let mut header_buf: [u8; 256] = [0x23; 256];
//         header_str.as_bytes().read(&mut header_buf);

//         writer.write(&header_buf);
//     }
//     while let Ok(n) = reader.read(&mut buf) {
//         if n == 0 {
//             break;
//         }
//         let _ = writer.write(&buf[..n]);
//     }
// }

fn header(host: &str) -> [u8; 256] {
    let mut header_str = format!("POST / HTTP/1.1\n");
    header_str += format!("Host: {}\n", host).as_str();
    header_str += format!("User-Agent: kurumi-proxy\n").as_str();
    header_str += format!("Accept: */*\n").as_str();
    header_str += format!("Content-Type: application/x-www-form-urlencoded\n").as_str();
    let mut header_buf: [u8; 256] = [0x23; 256];
    let mut _ptr_header_buf: &mut[u8] = &mut header_buf;
    _ptr_header_buf.write(header_str.as_bytes()).unwrap();

    return header_buf;
}

async fn proxy(input: &str, output: &str) -> io::Result<()> {
    let listener = TcpListener::bind(input).await?;
    loop {
        let (client, _) = listener.accept().await?;
        let server = TcpStream::connect(output).await?;

        let (mut client_read, mut client_write) = client.into_split();
        let (mut server_read, mut server_write) = server.into_split();

        server_write.write_all(&header(output)).await?;

        let client_to_server = tokio::spawn(async move {
            io::copy(&mut client_read, &mut server_write).await
        });
        let server_to_client = tokio::spawn(async move {
            io::copy(&mut server_read, &mut client_write).await
        });

        select!(
            _ = client_to_server => println!("c2s done."),
            _ = server_to_client => println!("s2c done."),
        )
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let kurumi = App::new("Kurumi")
        .version("0.1.0")
        .author("Sato Kaito <satodeyannsu@gmail.com>")
        .about("hide the TCP data from the F*CKING FW.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .required(true),
        )
        .get_matches();

    let input = kurumi.value_of("input").unwrap();
    let output = kurumi.value_of("output").unwrap();

    proxy(input, output).await
}
