use std::{env, net::Ipv4Addr};

use tiny_http::{Response, Server};

fn main() {
    let port = match env::var("HTTP_PORT") {
        Ok(port) => port
            .parse::<u16>()
            .expect("Failed to parse {port:?} as port number"),
        Err(env::VarError::NotPresent) => 8000,
        Err(e @ env::VarError::NotUnicode(..)) => panic!("{e}"),
    };
    let server = Server::http((Ipv4Addr::UNSPECIFIED, port)).expect("Couldn't listen for server");
    let code = match env::var("HTTP_CODE") {
        Ok(code) => code
            .parse::<u16>()
            .expect("Failed to parse {code:?} as status code"),
        Err(env::VarError::NotPresent) => 200,
        Err(e @ env::VarError::NotUnicode(..)) => panic!("{e}"),
    };
    println!("Responding with code {code}");
    loop {
        server
            .recv()
            .expect("Failed to receive")
            .respond(Response::empty(code))
            .expect("Failed to send response");
    }
}
