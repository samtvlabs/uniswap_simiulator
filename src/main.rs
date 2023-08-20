mod simulator;

use simulator::{fetch_historical_data, simulate_liquidity};
use clap::Parser;

fn get_subgraph_url(chain: &str) -> &'static str {
    match chain {
        "mainnet" => "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3",
        "rinkeby" => "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3-rinkeby",
        "ropsten" => "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3-ropsten",
        "kovan" => "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3-kovan",
        _ => panic!("Unsupported chain"),
    }
}

#[derive(clap::Parser)]
struct Opts {
    /// The EVM chain that Uniswap V3 is deployed to
    #[clap(short, long)]
    chain: String,

    /// The Uniswap V3 token pair address
    #[clap(short, long)]
    pair: String,

    /// The minimum price of the liquidity range
    #[clap(short, long)]
    min: i32,

    /// The maximum price of the liquidity range
    #[clap(short, long)]
    max: i32,

    /// The fee tier
    #[clap(short, long)]
    fee: f64,

    /// The start time of the simulation
    #[clap(short, long)]
    start: u64,

    /// The end time of the simulation
    #[clap(short, long)]
    end: u64,
}

fn main() {
    let opts: Opts = Opts::parse();

    let chain = &opts.chain;
    let pair = &opts.pair;
    let min = opts.min;
    let max = opts.max;
    let fee = opts.fee;
    let start = opts.start;
    let end = opts.end;

    let subgraph_url = get_subgraph_url(chain);
    let historical_data = fetch_historical_data(subgraph_url, pair, start, end);
    let simulation_results = simulate_liquidity(historical_data, min, max, fee);

    for result in simulation_results {
        println!("{:?}", result);
    }
}