use std::{net::TcpListener, thread::spawn};
use tungstenite::Message::Text;
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};

fn main() {
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |req: &Request, response: Response| {
                println!("Received a new ws handshake");
                println!("The request's path is: {}", req.uri().path());
                Ok(response)
            };
            let mut ws = accept_hdr(stream.unwrap(), callback).unwrap();

            loop {
                let msg = ws.read().unwrap();
                match msg {
                    Text(str) => {
                        println!("message received: {}", str);
                        let response = String::from("Hello from tungstenite");
                        ws.send(Text(response)).unwrap();
                    },
                    _ => (),
                }
            }
        });
    }
}
