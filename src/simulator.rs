use reqwest;
use graphql_client::GraphQLQuery;
use serde_json::Value;

pub type BigInt = f64;
pub type BigDecimal = f64;


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema/schema.graphql",
    query_path = "src/queries/get_pool_hour_data.graphql",
    response_derives = "Debug"
)]
struct GetPoolHourData;


pub struct SimulationResult {
    date: i64,
    reserves_token1: f64,
    reserves_token2: f64,
    accumulated_fees: f64,
}

impl std::fmt::Debug for SimulationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimulationResult").field("date", &self.date).field("reserves_token1", &self.reserves_token1).field("reserves_token2", &self.reserves_token2).field("accumulated_fees", &self.accumulated_fees).finish()
    }
}
pub fn fetch_historical_data(
    subgraph_url: &str,
    pool_id: &str,
    start_timestamp: u64,
    end_timestamp: u64,
) -> Vec<get_pool_hour_data::GetPoolHourDataPoolHourDatas> {
    let variables = get_pool_hour_data::Variables {
        pool: pool_id.to_string(),
        period_start_unix_gt: start_timestamp as i64,
        period_start_unix_lt: end_timestamp as i64,
    };
    let client = reqwest::blocking::Client::new();
    let response = client.post(subgraph_url).json(&GetPoolHourData::build_query(variables))
    .send();
   

    match response {
        Ok(resp) => {
            let text_resp = resp.text().unwrap();
            // Attempt to pretty print JSON
            let pretty_json: Result<Value, _> = serde_json::from_str(&text_resp);
            match pretty_json {
                Ok(json) => println!("response: {}", serde_json::to_string_pretty(&json).unwrap()),
                Err(_) => println!("Raw Response Text: {}", text_resp),
            }
            match serde_json::from_str::<graphql_client::Response<get_pool_hour_data::ResponseData>>(&text_resp) {
                Ok(data) => {
                    if let Some(pool_hour_datas) = data.data.and_then(|data| Some(data.pool_hour_datas)) {
                        pool_hour_datas
                    } else {
                        vec![]
                    }
                },
                Err(err) => {
                    println!("Failed to deserialize response: {:?}", err);
                    vec![]
                }
            }
        },
        Err(err) => {
            println!("Request failed: {:?}", err);
            vec![]
        }
    }
}

pub fn simulate_liquidity(
    historical_data: Vec<get_pool_hour_data::GetPoolHourDataPoolHourDatas>,
    lower_tick: Option<f64>,
    upper_tick: Option<f64>,
    fee_tier: f64,
) -> Vec<SimulationResult> {
    let mut accumulated_fees = 0.0;
    let mut reserves_token1 = 0.0;
    let mut reserves_token2 = 0.0;
    let mut results = vec![];

    for entry in historical_data {
        if let (Some(lower), Some(upper)) = (lower_tick, upper_tick) {
            if entry.tick >= Some(lower) && entry.tick <= Some(upper) {
                accumulated_fees += entry.fees_usd * fee_tier;
                reserves_token1 += entry.volume_usd / entry.sqrt_price;
                reserves_token2 += entry.volume_usd * entry.sqrt_price;
    
                results.push(SimulationResult {
                    date: entry.period_start_unix,
                    reserves_token1,
                    reserves_token2,
                    accumulated_fees,
                });
            }
        }
        
    }

    results
}
