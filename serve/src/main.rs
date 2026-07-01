use std::net::TcpListener;
use std::thread::spawn;

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr)?;
    println!("Serving at http://{}", addr);
    println!("Open http://127.0.0.1:8080 in your browser");
    println!("Press Ctrl+C to stop");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        spawn(|| handle(stream));
    }
    Ok(())
}

fn handle(mut stream: std::net::TcpStream) {
    use std::io::{BufRead, BufReader, Write};

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let path = if path == "/" { "/index.html" } else { path };
    let path = path.trim_start_matches('/');

    let mime = match path.split('.').last() {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("md") => "text/plain",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };

    let raw = std::fs::read(&path);
    let (status_line, body) = match raw {
        Ok(data) => ("HTTP/1.1 200 OK", data),
        Err(_) => ("HTTP/1.1 404 NOT FOUND", b"404 Not Found".to_vec()),
    };

    let header = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        mime,
        body.len()
    );

    let mut response = header.as_bytes().to_vec();
    response.extend_from_slice(&body);
    let _ = stream.write_all(&response);
}
