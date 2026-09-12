mod capture;
mod db;
mod detect;
mod models;
mod parser;
mod protocols;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--list" || a == "-l") {
        return capture::list_interfaces();
    }

    let pool = db::init_pool().await?;
    let tx = db::spawn_writer(pool);

    if args.iter().any(|a| a == "--demo") {
        detect::run_demo_loop(tx).await;
        return Ok(());
    }

    let env_device = std::env::var("PCAP_DEVICE").ok();
    let preferred = args
        .iter()
        .position(|a| a == "--iface" || a == "-i")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .or(env_device.as_deref());

    let interface = capture::resolve_interface(preferred)?;
    println!("Starting OT Security Monitor...");
    println!("Capture interface: {interface}");
    println!("Tip: cargo run -- --demo   (sample Modbus/OPC UA events)");
    println!("     cargo run -- --list   (show interfaces)");

    let iface = interface.clone();
    let capture_tx = tx.clone();
    tokio::task::spawn_blocking(move || capture::start_capture(&iface, capture_tx)).await??;

    Ok(())
}
