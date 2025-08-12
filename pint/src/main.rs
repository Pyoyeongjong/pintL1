use clap::Parser;
use node::{builder::LaunchContext, error::LaunchError, node::PintNode};
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

#[tokio::main]
async fn main() -> Result<(), LaunchError>{

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }
    
    let args = Args::parse();
    let address = args.address;
    let port = args.port;

    // Ensure DB is imported from the appropriate module
    let components_builder = PintNode::components::<PintNode>();
    let ctx = LaunchContext { address, port, components_builder };

    ctx.launch().await?;
    Ok(())
}