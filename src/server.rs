use hyper::{Body, Request, Response, Server, service::{make_service_fn, service_fn}};
use prometheus::{Registry, Encoder, TextEncoder};
use std::convert::Infallible;
use std::net::SocketAddr;

pub async fn run_server(
    registry: Registry,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let make_svc = make_service_fn(move |_conn| {
        let registry = registry.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let registry = registry.clone();
                async move {
                    if req.uri().path() == "/metrics" {
                        let encoder = TextEncoder::new();
                        let metric_families = registry.gather();
                        let mut buffer = Vec::new();
                        encoder.encode(&metric_families, &mut buffer).unwrap();

                        Ok::<_, Infallible>(
                            Response::builder()
                                .status(200)
                                .header("Content-Type", encoder.format_type())
                                .body(Body::from(buffer))
                                .unwrap()
                        )
                    } else {
                        Ok::<_, Infallible>(
                            Response::builder()
                                .status(404)
                                .body(Body::from("Not Found"))
                                .unwrap()
                        )
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Serving Prometheus metrics on http://{}", addr);

    server.await?;
    Ok(())
}
