use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, Read, Write, BufReader, BufWriter};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    for stream in listener.incoming() {
      match stream {
        Ok(stream) => {
            handle_request(stream);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
      }
    }
}

fn handle_request(mut stream: TcpStream) {
  let mut reeader = BufReader::new(stream.try_clone().unwrap());
  let mut writer = BufWriter::new(&mut stream);

  // let http_request: Vec<String> = buf_reader
  //   .lines()
  //   .map(|result| result.unwrap())
  //   .take_while(|line| !line.is_empty())
  //   .collect();

  // println!("{:#?}", http_request);

  // if &http_request[0] == "GET / HTTP/1.1" {  
  //   // Send the HTTP response headers
  // }
  let response_headers = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nTransfer-Encoding: chunked\r\n\r\n";
  writer.write_all(response_headers.as_bytes()).unwrap();
  writer.flush().unwrap();

  // Streaming HTML response
  let html_start = "<!DOCTYPE html><html><body><h1>Streaming HTML Response</h1>";
  writer.write_all(format!("{:X}\r\n", html_start.len()).as_bytes()).unwrap();
  writer.write_all(html_start.as_bytes()).unwrap();
  writer.write_all(b"\r\n").unwrap();

  for i in 0..10 {
    let chunk = format!("Chunk {}<br>", i);
    writer.write_all(format!("{:X}\r\n", chunk.len()).as_bytes()).unwrap();
    writer.write_all(chunk.as_bytes()).unwrap();
    writer.write_all(b"\r\n").unwrap();
    writer.flush().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200)); // simulate some delay
  }

  let html_end = "</body></html>";
  writer.write_all(format!("{:X}\r\n", html_end.len()).as_bytes()).unwrap();
  writer.write_all(html_end.as_bytes()).unwrap();
  writer.write_all(b"\r\n0\r\n\r\n").unwrap();
  writer.flush().unwrap();
}