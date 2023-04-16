use clap::Parser;
use reqwest::header;
// use std::io::{self, Read, Write};
use std::str;
use std::io::{Read, Write};
use tokio::io::{self, AsyncWriteExt, BufReader, AsyncReadExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;


fn header(host: &str) -> [u8; 256] {
    let mut header_str = format!("POST / HTTP/1.1\n");
    header_str += format!("Host: {}\n", host).as_str();
    header_str += format!("User-Agent: kurumi-proxy\n").as_str();
    header_str += format!("Accept: */*\n").as_str();
    header_str += format!("Content-Type: application/x-www-form-urlencoded\n").as_str();
    let mut header_buf: [u8; 256] = [0x20; 256];
    let mut _ptr_header_buf: &mut[u8] = &mut header_buf;
    _ptr_header_buf.write(header_str.as_bytes()).unwrap();

    return header_buf;
}

fn response_header() ->[u8; 256] {
    let mut header_str = format!("HTTP/1.1 200 OK");
    header_str += format!("Server: kurumi_cracker").as_str();
    header_str += format!("Connection: Keep-Alive").as_str();
    header_str += format!("Content-Type: text/html").as_str();
    let mut header_buf: [u8; 256] = [0x20; 256];
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

        let mut buffer = vec![0; 256];
        server_read.read_exact(&mut buffer).await?;

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


async fn kurumi_cracker(input: &str, output: &str) -> io::Result<()> {
    let listener = TcpListener::bind(input).await?;
    loop {
        let (client, _) = listener.accept().await?;
        let server = TcpStream::connect(output).await?;

        // let mut stream = BufReader::new(client);
        // let mut buffer = String::new();
        // for i in 0..4 {
        //     stream.read_line(&mut buffer).await.unwrap();
        //     println!("{}", buffer);
        // }

        let (mut client_read, mut client_write) = client.into_split();
        let (mut server_read, mut server_write) = server.into_split();

        client_write.write_all(&response_header()).await?;

        let mut buffer = vec![0; 256];
        client_read.read_exact(&mut buffer).await?;

        let client_to_server = tokio::spawn(async move {
            io::copy(&mut client_read, &mut server_write).await
            // let mut buf = vec![0; 1024];

            // loop {
            //     match client_read.read(&mut buf).await {
            //         Ok(0) => return,
            //         Ok(n) => {
            //             if server.write_all()
            //         }
            //     }
            // }
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

#[derive(Parser)]
struct Args {
    mode: String, // kurumi or kurumi_cracker
    client: String,
    server: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let mode = args.mode;
    match &*mode {
        "client" => proxy(&args.client, &args.server).await,
        "server" => kurumi_cracker(&args.client, &args.server).await,
        _ => Ok(()),
    }
}
