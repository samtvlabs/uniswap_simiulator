use crate::thegraph;
use std::str::FromStr;

pub enum DataSource {
    // RPC,
    TheGraph,
    // Reth,
}

impl FromStr for DataSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "thegraph" => Ok(DataSource::TheGraph),
            _ => Err(format!("Unknown data source: {}", s)),
        }
    }
}

pub trait DataClient {
    fn get_pool_data(&self, subgraph_url: &str, id: &str) -> Option<thegraph::get_pool::GetPoolPool>;
}