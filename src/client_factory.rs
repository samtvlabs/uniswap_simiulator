use crate::data_source::{DataClient, DataSource};
use crate::thegraph::TheGraphClient;
// Import other clients...

pub fn create_client(data_source: &DataSource) -> Box<dyn DataClient> {
    match data_source {
        // DataSource::RPC => Box::new(RPCClient::new()),
        DataSource::TheGraph => Box::new(TheGraphClient::new()),
        // DataSource::Reth => Box::new(RethClient::new()),
    }
}