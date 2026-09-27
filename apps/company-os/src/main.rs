use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn response_body() -> String {
    let agents = [
        ("Governor", "READY"),
        ("CEO", "READY"),
        ("CFO", "READY"),
        ("COO", "READY"),
        ("Analyst", "READY"),
        ("Experiment", "READY"),
        ("Growth", "READY"),
        ("Content", "READY"),
        ("Recruiter", "READY"),
    ];

    let rows = agents
        .iter()
        .map(|(name, status)| format!("<tr><td>{name}</td><td>{status}</td></tr>"))
        .collect::<Vec<_>>()
        .join("");

    format!(r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Company OS</title>
<style>
body{{font-family:system-ui,sans-serif;max-width:1100px;margin:40px auto;padding:0 20px}}
.card{{border:1px solid #ddd;border-radius:12px;padding:18px;margin:16px 0}}
table{{width:100%;border-collapse:collapse}}th,td{{text-align:left;padding:10px;border-bottom:1px solid #eee}}
.status{{font-weight:600}}
</style>
</head>
<body>
<h1>Company OS</h1>
<p>Rust runtime • bootstrap control plane</p>
<div class="card"><strong>Economic Core</strong><p>Loaded and enforced by deterministic Rust code.</p></div>
<div class="card"><strong>Agents</strong><table><tr><th>Agent</th><th>Status</th></tr>{rows}</table></div>
</body>
</html>"#)
}

fn handle(mut stream: TcpStream) {
    let mut buffer = [0_u8; 4096];
    let _ = stream.read(&mut buffer);

    let request = String::from_utf8_lossy(&buffer);
    let path = request.split_whitespace().nth(1).unwrap_or("/");

    if path == "/healthz" {
        let body = "ok
";
        let response = format!(
            "HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8
Content-Length: {}
Connection: close

{}",
            body.len(), body
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    if path != "/" {
        let body = "not found
";
        let response = format!(
            "HTTP/1.1 404 Not Found
Content-Type: text/plain; charset=utf-8
Content-Length: {}
Connection: close

{}",
            body.len(), body
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let body = response_body();
    let response = format!(
        "HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Content-Length: {}
Connection: close

{}",
        body.len(), body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn main() {
    let listener = TcpListener::bind(("0.0.0.0", 8080)).expect("failed to bind port 8080");
    println!("Company OS listening on http://localhost:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle(stream));
            }
            Err(error) => eprintln!("connection error: {error}"),
        }
    }
}
