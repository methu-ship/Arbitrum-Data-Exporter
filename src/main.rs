use ethers::providers::{Provider, Http};
use ethers::middleware::Middleware; 
use prometheus::{Encoder, Gauge, TextEncoder, Registry};
use std::convert::Infallible;
use std::env;
use hyper::{Server, Request, Response, Body, service::{make_service_fn, service_fn}};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gas_metric = Gauge::new("arbitrum_gas_price_gwei", "Current Arbitrum Gas Price in Gwei")?;
    let block_metric = Gauge::new("arbitrum_latest_block", "Latest Arbitrum Block Number")?;
    let registry = Registry::new();
    registry.register(Box::new(gas_metric.clone()))?;
    registry.register(Box::new(block_metric.clone()))?;


    let rpc_url = env::var("RPC_URL")?;
    let provider = Provider::<Http>::try_from(rpc_url)?;

    let provider_metrics = provider.clone();
    let gas_metric_clone = gas_metric.clone();
    let block_metric_clone = block_metric.clone();

    
    let update_metrics = async move { // Async loop to update Prometheus metrics every 15 seconds
        loop {
            if let Ok(block) = provider_metrics.get_block_number().await {
                block_metric_clone.set(block.as_u64() as f64);
            }
            if let Ok(gas_price) = provider_metrics.get_gas_price().await {
                let gas_gwei = gas_price.as_u64() as f64 / 1e9;
                gas_metric_clone.set(gas_gwei);
            }
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        }
        #[allow(unreachable_code)]
        Ok::<(), Box<dyn std::error::Error>>(())
    };
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

   
    let addr = ([0, 0, 0, 0], 8000).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Serving Prometheus metrics on http://{}", addr);

    
    tokio::try_join!(
        update_metrics,    // Run both the metrics update loop and HTTP server concurrently
        async {
            if let Err(e) = server.await {
                eprintln!("Server error: {}", e);
                Err(Box::new(e) as Box<dyn std::error::Error>)
            } else {
                Ok(())
            }
        }
    )?;

    Ok(())
}
