mod thegraph;
mod client_factory;
mod data_source;

use client_factory::create_client;
use data_source::DataSource;

use clap::Parser;

fn get_subgraph_url(chain: &str) -> &'static str {
    match chain {
        "ethereum" => "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3",
        "polygon" => "https://api.thegraph.com/subgraphs/name/ianlapham/uniswap-v3-polygon",
        "celo" => "https://api.thegraph.com/subgraphs/name/jesse-sawa/uniswap-celo",
        "optimism" => "https://api.thegraph.com/subgraphs/name/ianlapham/optimism-post-regenesis",
        "arbitrum" => "https://api.thegraph.com/subgraphs/name/ianlapham/arbitrum-minimal",
        "bnb" => "https://api.thegraph.com/subgraphs/name/ianlapham/uniswap-v3-bsc",
        _ => panic!("Unsupported chain"),
    }
}

#[derive(clap::Parser)]
struct Opts {
    #[clap(short, long)]
    chain: String,

    /// The Uniswap V3 token pair address
    #[clap(short, long)]
    pair: String,

    /// Data source (TheGraph, RPC, Reth, etc.)
    #[clap(short, long)]
    data_source: String,
}

fn parse_data_source(source: &str) -> Option<DataSource> {
    match source.to_lowercase().as_str() {
        "thegraph" => Some(DataSource::TheGraph),
        // Add other sources here
        _ => None,
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    let chain = &opts.chain;
    let pair = &opts.pair;

    if let Some(data_source) = parse_data_source(&opts.data_source) {
        let client = create_client(&data_source); // Pass a reference to avoid moving the value
        match &data_source { 
            DataSource::TheGraph => {
                let subgraph_url = get_subgraph_url(chain);
                let pool_data = client.get_pool_data(subgraph_url, pair);
                match pool_data {
                    Some(data) => {
                        // Process the data as needed
                        println!("Pool data: {:?}", data);
                    },
                    None => {
                        println!("No data found for pool {}", pair);
                    }
                }
            },
            // Handle other data sources as needed
        }
    } else {
        eprintln!("Invalid data source provided: {}", opts.data_source);
    }
}

