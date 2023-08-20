use reqwest;
use serde::Deserialize;
// use serde_json::Value;
use graphql_client::GraphQLQuery;

#[derive(Deserialize, Debug)]
struct PoolHourData {
    periodStartUnix: u64,
    tick: i32,
    volumeUSD: f64,
    feesUSD: f64,
    sqrtPrice: f64,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema/schema.graphql",
    query_path = "src/queries/get_pool_hour_data.graphql",
    response_derives = "Debug",
    operation_name = "GetPoolHourData"
)]
struct GetPoolHourData;

#[derive(Debug)]
struct SimulationResult {
    date: u64,
    reserves_token1: f64,
    reserves_token2: f64,
    accumulated_fees: f64,
}

pub fn fetch_historical_data(
    subgraph_url: &str,
    pool_id: &str,
    start_timestamp: u64,
    end_timestamp: u64,
) -> Vec<get_pool_hour_data::PoolHourData> {
    let variables = get_pool_hour_data::Variables {
        pool: pool_id.to_string(),
        periodStartUnix_gt: start_timestamp,
        periodStartUnix_lt: end_timestamp,
    };
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(subgraph_url)
        .json(&GetPoolHourData::build_query(variables))
        .send()
        .unwrap()
        .json::<graphql_client::Response<get_pool_hour_data::ResponseData>>()
        .unwrap();

    if let Some(pool_hour_datas) = response.data.and_then(|data| data.pool_hour_datas) {
        pool_hour_datas
    } else {
        vec![]
    }
}

pub fn simulate_liquidity(
    historical_data: Vec<PoolHourData>,
    lower_tick: i32,
    upper_tick: i32,
    fee_tier: f64,
) -> Vec<SimulationResult> {
    let mut accumulated_fees = 0.0;
    let mut reserves_token1 = 0.0;
    let mut reserves_token2 = 0.0;
    let mut results = vec![];

    for entry in historical_data {
        if entry.tick >= lower_tick && entry.tick <= upper_tick {
            accumulated_fees += entry.feesUSD * fee_tier;
            reserves_token1 += entry.volumeUSD / entry.sqrtPrice;
            reserves_token2 += entry.volumeUSD * entry.sqrtPrice;

            results.push(SimulationResult {
                date: entry.periodStartUnix,
                reserves_token1,
                reserves_token2,
                accumulated_fees,
            });
        }
    }

    results
}
