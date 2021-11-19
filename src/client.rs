use std::{str::FromStr, thread::sleep, time::Duration};

use hyper::{Client, StatusCode, body::HttpBody};
use wireguard_control::{Device, InterfaceName, Backend, DeviceUpdate};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(help = "the interface name of wireguard")]
    device_name: String,

    #[structopt(help = "the address of takepoint server")]
    server_addr: String,

    #[structopt(help = "the list of peers behind NAT")]
    peer_list: Vec<String>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let interface_name = InterfaceName::from_str(opt.device_name.as_str()).unwrap();
    let backend = Backend::default();
    let device = Device::get(&interface_name, backend).unwrap();

    let client = Client::new();
    loop {
        let mut update = DeviceUpdate::new();

        for peer in device.peers.iter() {
            let pubkey = peer.config.public_key.to_base64();
            if !opt.peer_list.contains(&pubkey) {
                continue;
            }

            let uri = format!("http://{}/{}", opt.server_addr, pubkey).parse().unwrap();
            let mut resp = client.get(uri).await.unwrap();

            if resp.status() == StatusCode::OK {
                let body = resp.body_mut().data().await.unwrap().unwrap();
                let endpoint = String::from_utf8(body.to_vec()).unwrap();
                
                update = update.add_peer_with(&peer.config.public_key, |peer| {
                    println!("updating endpoint for {} to {}", &pubkey, endpoint);
                    peer.set_endpoint(endpoint.parse().unwrap())
                });
            }
        }

        update.apply(&interface_name, backend).unwrap();
        sleep(Duration::from_secs(15));
    }
}
