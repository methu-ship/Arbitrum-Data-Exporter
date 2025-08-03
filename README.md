# Arbitrum Gas & Block Metrics Exporter

A simple Rust Prometheus exporter that periodically fetches the latest Arbitrum block number and gas price (in Gwei) from an Arbitrum node RPC endpoint, and exposing these as Prometheus metrics for monitoring and visualization.

---

## Features

- Connects to Arbitrum node via JSON-RPC (RPC URL set by `RPC_URL` environment variable)
- Collects two metrics:
  - `arbitrum_latest_block`: latest block number on Arbitrum
  - `arbitrum_gas_price_gwei`: current gas price converted to Gwei
- Updates metrics every 15 seconds asynchronously
- Exposes `/metrics` HTTP endpoint on port 8000 for Prometheus scraping
- Easily integrates with Prometheus and Grafana dashboards

---

## Prerequisites

- Rust toolchain installed (stable recommended)
- Running Arbitrum node with RPC HTTP endpoint accessible
- Prometheus server (to scrape metrics)
- Grafana (optional) for visualization

---

## Setup & Usage

1. **Clone the repository**
git clone <your-repo-url>
cd arb_monitoring


2. **Set the Arbitrum RPC URL environment variable**
export RPC_URL="http://localhost:8547" # Replace with your Arbitrum node RPC URL


3. **Build and run the exporter**
cargo run

## Prometheus Configuration
Download Prometheus and the following scrape config to your `prometheus.yml`:

You can then verify scraping via Prometheus web UI: [http://localhost:9090/targets]
