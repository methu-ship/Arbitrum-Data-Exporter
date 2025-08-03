use prometheus::{Gauge, Registry};

pub struct ArbitrumMetrics {
    pub gas_price_gauge: Gauge,
    pub latest_block_gauge: Gauge,
    pub registry: Registry,
}

impl ArbitrumMetrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let gas_price_gauge = Gauge::new("arbitrum_gas_price_gwei", "Current Arbitrum Gas Price in Gwei")?;
        let latest_block_gauge = Gauge::new("arbitrum_latest_block", "Latest Arbitrum Block Number")?;

        let registry = Registry::new();
        registry.register(Box::new(gas_price_gauge.clone()))?;
        registry.register(Box::new(latest_block_gauge.clone()))?;

        Ok(Self {
            gas_price_gauge,
            latest_block_gauge,
            registry,
        })
    }
}
