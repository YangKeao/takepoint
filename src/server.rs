use std::future;
use std::str::FromStr;
use std::convert::Infallible;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use wireguard_control::{Device, InterfaceName, Backend};
use structopt::StructOpt;

#[derive(StructOpt, Clone)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(help = "the interface name of wireguard")]
    device_name: String,

    #[structopt(help = "the listening address of this service")]
    listen_addr: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let addr = opt.listen_addr.parse().unwrap();

    let make_svc = make_service_fn(move |_conn| {
        let opt = opt.clone();

        async {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| -> future::Ready<Result<_, Infallible>> {
                let mut response = Response::new(Body::empty());

                if req.uri().path().len() < 1 {
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return future::ready(Ok(response))
                }

                let pubkey = &req.uri().path()[1..];
    
                let interface_name = InterfaceName::from_str(opt.device_name.as_str()).unwrap();
                let backend = Backend::default();
                let device = Device::get(&interface_name, backend).unwrap();
                for peer in device.peers.iter() {
                    if peer.config.public_key.to_base64() == pubkey {
                        match peer.config.endpoint {
                            Some(endpoint) => {
                                *response.body_mut() = Body::from(endpoint.to_string());
                            }
                            None => {
                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            }
                        }
    
                        return future::ready(Ok(response))
                    }
                }
                
                *response.status_mut() = StatusCode::NOT_FOUND;
                future::ready(Ok(response))
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}