use clap::Parser;
use node::{builder::LaunchContext, node::PintNode};
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address
    #[arg(short, long)]
    address: IpAddr,

    /// Port
    #[arg(short, long, default_value_t = 8557)]
    port: u16,
}

fn main() {
    let args = Args::parse();
    let address = args.address;
    let port = args.port;

    let builder = PintNode::components();

    let ctx = LaunchContext { address, port };

    ctx.launch();
}
