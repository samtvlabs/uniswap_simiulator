use crate::data_source::DataClient;
use graphql_client::GraphQLQuery;
use reqwest;
use serde_json;

pub struct TheGraphClient;

impl TheGraphClient {
    pub fn new() -> Self {      
        Self
    }
}



pub type BigInt = String;
pub type BigDecimal = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema/schema.graphql",
    query_path = "src/queries/get_pool.graphql",
    response_derives = "Debug"
)]
pub struct GetPool;

impl DataClient for TheGraphClient {
    fn get_pool_data(
        &self,
        subgraph_url: &str,
        id: &str
    ) -> Option<get_pool::GetPoolPool> {
        let variables = get_pool::Variables {
            id: id.to_string(),
        };
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(subgraph_url)
            .json(&GetPool::build_query(variables))
            .send();
    
        match response {
            Ok(resp) => {
                let text_resp = resp.text().unwrap();
                match serde_json::from_str::<graphql_client::Response<get_pool::ResponseData>>(&text_resp) {
                    Ok(data) => {
                        // Extract the specific pool data
                        data.data.and_then(|data| data.pool)
                    },
                    Err(err) => {
                        println!("Failed to deserialize response: {:?}", err);
                        None
                    }
                }
            },
            Err(err) => {
                println!("Request failed: {:?}", err);
                None
            }
        }
    }
}


