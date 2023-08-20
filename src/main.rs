use reqwest;
use serde::Deserialize;
use serde_json::Value;
use  graphql_client::GraphQLQuery;

const BASE_URL: &str = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3";

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
    response_derives = "Debug"
)]
struct GetPoolHourData;

#[derive(Debug)]
struct SimulationResult {
    date: u64,
    reserves_token1: f64,
    reserves_token2: f64,
    accumulated_fees: f64,
}

// fn fetch_historical_data(
//     pool_id: &str,
//     start_timestamp: u64,
//     end_timestamp: u64,
// ) -> Vec<PoolHourData> {
//     let client = reqwest::blocking::Client::new();

//     let query = format!(
//         r#"
//     {{
//       poolHourDatas(first: 1000, where:{{
//         pool: "{}",
//         periodStartUnix_gt: {},
//         periodStartUnix_lt: {}
//       }}) {{
//         periodStartUnix
//         tick
//         volumeUSD
//         feesUSD
//         sqrtPrice
//       }}
//     }}
//     "#,
//         pool_id, start_timestamp, end_timestamp
//     );

//     println!("{}", query);

//     let response: Value = client
//         .post(BASE_URL)
//         .header("Content-Type", "application/json")
//         .body(format!(r#"{{"query":"{}"}}"#, query))
//         .send()
//         .unwrap()
//         .json()
//         .unwrap();
//     println!("{}", response);
//     response["data"]["poolHourDatas"]
//         .as_array()
//         .unwrap()
//         .iter()
//         .map(|x| serde_json::from_value(x.clone()).unwrap())
//         .collect()
// }

fn fetch_historical_data(
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
    let response = client.post(BASE_URL)
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

fn simulate_liquidity(
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

fn main() {
    let pool_id = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8"; // This is the pool ID for WETH-USDC pair on Ethereum mainnet.
    let start_timestamp = 1672444800; // This is 15 August 2021 @ 12:00am (UTC)
    let end_timestamp = 1677705600; // This is 15 September 2021 @ 12:00am (UTC)

    let lower_tick = -60000; // Hypothetical lower tick value for the liquidity range
    let upper_tick = 60000; // Hypothetical upper tick value for the liquidity range
    let fee_tier = 0.003; // 0.30% fee tier. Remember, this should be in decimal form.

    let historical_data = fetch_historical_data(&pool_id, start_timestamp, end_timestamp);
    let simulation_results = simulate_liquidity(historical_data, lower_tick, upper_tick, fee_tier);

    for result in simulation_results {
        println!("{:?}", result);
    }
}
