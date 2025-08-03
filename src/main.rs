use ethers::providers::{Provider, Http};
use ethers::middleware::Middleware; 
use prometheus::{Encoder, Gauge, TextEncoder, Registry};
use std::convert::Infallible;
use std::env;
use hyper::{Server, Request, Response, Body, service::{make_service_fn, service_fn}};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Prometheus gauges for gas price and latest block
    let gas_metric = Gauge::new("arbitrum_gas_price_gwei", "Current Arbitrum Gas Price in Gwei")?;
    let block_metric = Gauge::new("arbitrum_latest_block", "Latest Arbitrum Block Number")?;

    // Registry to hold the metrics
    let registry = Registry::new();
    registry.register(Box::new(gas_metric.clone()))?;
    registry.register(Box::new(block_metric.clone()))?;

    // Read Arbitrum RPC URL from environment variable named "RPC_URL"
    let rpc_url = env::var("RPC_URL")?;

    // Connect to Arbitrum node provider
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // Clone providers and metrics for use in update loop
    let provider_metrics = provider.clone();
    let gas_metric_clone = gas_metric.clone();
    let block_metric_clone = block_metric.clone();

    // Async loop to update Prometheus metrics every 15 seconds
    let update_metrics = async move {
        loop {
            if let Ok(block) = provider_metrics.get_block_number().await {
                block_metric_clone.set(block.as_u64() as f64);
            }
            if let Ok(gas_price) = provider_metrics.get_gas_price().await {
                // Convert wei to gwei
                let gas_gwei = gas_price.as_u64() as f64 / 1e9;
                gas_metric_clone.set(gas_gwei);
            }
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        }
        // technically unreachable, but required for type compatibility
        #[allow(unreachable_code)]
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    // Create HTTP service to expose the /metrics endpoint for Prometheus scraping
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

    // Define socket address, listen on all interfaces port 8000
    let addr = ([0, 0, 0, 0], 8000).into();

    // Build server future
    let server = Server::bind(&addr).serve(make_svc);

    println!("Serving Prometheus metrics on http://{}", addr);

    // Run both the metrics update loop and HTTP server concurrently
    tokio::try_join!(
        update_metrics,
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
