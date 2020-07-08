use futures::stream::TryStreamExt;
use network_bridge::BridgeBuilder;

use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error, Handle};
use tokio;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bridge_name = "rucker0";
    let bridge = BridgeBuilder::new(bridge_name).build();
    match bridge {
        Ok(_brg) => println!("{} is created!", bridge_name),
        Err(err) => println!("Error: {}", err),
    }

    let link_name = bridge_name;
    let ip_str = "172.29.0.1/16";
    let ip: IpNetwork = ip_str.parse().unwrap_or_else(|_| {
        eprintln!("invalid address");
        std::process::exit(1);
    });

    // ref: https://tech-blog.optim.co.jp/entry/2019/11/08/163000#tokio
    let mut rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (connection, handle, _) = new_connection().unwrap();
        // ↓は何? ↑のconnection は何?
        tokio::spawn(connection);

        if let Err(e) = add_address(link_name, ip, handle.clone()).await {
            eprintln!("{}", e);
        }
        // TODO: `ip link set rucker up` ?
        Ok(())
    })
}

fn create_container_id() -> String {
    // TODO: implement
    String::from("THIS_IS_PLACEHOLDER_CONTAINER_ID")
}

fn init_container() {
    let contianer_id = create_container_id();
    println!("New container ID: {}\n", contianer_id);
    // TODO: implement image related processes
}

fn prepare_and_execute_container(
    mem: i32,
    swap: i32,
    pids: i32,
    cpus: f64,
    container_id: &str,
    image_sha_hex: &str,
    cmd_args: Vec<&str>,
) {
    /* Setup the network namespace */
}

async fn add_address(link_name: &str, ip: IpNetwork, handle: Handle) -> Result<(), Error> {
    let mut links = handle
        .link()
        .get()
        .set_name_filter(link_name.to_string())
        .execute();

    // ???: what is `links.try_next()`
    // ref: https://crates.io/crates/futures
    if let Some(link) = links.try_next().await? {
        handle
            .address()
            .add(link.header.index, ip.ip(), ip.prefix())
            .execute()
            .await?
    }
    Ok(())
}
