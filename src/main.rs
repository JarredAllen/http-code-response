use http_testing_server::ServerResponder;

use clap::Parser;
use tiny_http::Server;

use std::{net::Ipv4Addr, path::PathBuf, time::Duration};

#[derive(Parser)]
struct Args {
    /// The directory to host, if any
    host_directory: Option<PathBuf>,
    #[arg(short, long, default_value_t = 8000)]
    port: u16,
    /// The status code to return.
    #[arg(long)]
    status_code: Option<u16>,
    /// An extra delay to add before responding, in seconds
    #[arg(long, default_value_t = 0)]
    extra_delay: u64,
}

fn main() {
    let args = Args::parse();
    let server =
        Server::http((Ipv4Addr::UNSPECIFIED, args.port)).expect("Couldn't listen for server");
    let responder = {
        let mut builder = ServerResponder::builder();
        if let Some(host_directory) = args.host_directory {
            builder = builder.host_directory(host_directory);
        }
        if let Some(status_code) = args.status_code {
            builder = builder.status_code(status_code);
        }
        builder
            .extra_delay(Duration::from_secs(args.extra_delay))
            .build()
    };
    loop {
        let request = server.recv().expect("Failed to receive");
        let response = responder.respond(&request);
        request.respond(response).expect("Failed to send response");
    }
}
