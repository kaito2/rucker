use network_bridge::BridgeBuilder;

fn main() {
    let bridge_name = "rucker0";
    let bridge = BridgeBuilder::new(bridge_name).build();
    match bridge {
        Ok(_brg) => println!("{} is created!", bridge_name),
        Err(err) => println!("Error: {}", err),
    }
}
