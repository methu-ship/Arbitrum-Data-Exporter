use ethers::middleware::Middleware;
use prometheus::Gauge;
use tokio::time::{sleep, Duration};

pub async fn run_updater<M: Middleware + Clone + Send + Sync + 'static>(
    provider: M,
    gas_price_gauge: Gauge,
    latest_block_gauge: Gauge,
) {
    loop {
        if let Ok(block) = provider.get_block_number().await {
            latest_block_gauge.set(block.as_u64() as f64);
        }

        if let Ok(gas_price) = provider.get_gas_price().await {
            let gas_gwei = gas_price.low_u64() as f64 / 1e9;
            gas_price_gauge.set(gas_gwei);
        }

        sleep(Duration::from_secs(15)).await;
    }
}
