use graphql_client::GraphQLQuery;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use num_traits::{pow, Float, One, Zero};
use reqwest;
use serde_json::Value;
use std::str::FromStr;

pub type BigInt = String;
pub type BigDecimal = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema/schema.graphql",
    query_path = "src/queries/get_pool_hour_data.graphql",
    response_derives = "Debug"
)]
struct GetPoolHourData;

// #[derive(GraphQLQuery)]
// #[graphql(
//     schema_path = "src/schema/schema.graphql",
//     query_path = "src/queries/get_pool.graphql",
//     response_derives = "Debug"
// )]
// struct GetPool;

#[derive(Debug)]
pub struct Network {
    id: String,
    chain_id: u32,
    name: String,
    desc: String,
    logo_uri: String,
    disabled: Option<bool>,
    is_new: Option<bool>,
    error: Option<String>,
    subgraph_endpoint: String,
    total_value_locked_usd_gte: f64,
    volume_usd_gte: f64,
    disabled_top_positions: Option<bool>,
}

#[derive(Debug)]
pub struct Tick {
    tick_idx: String,
    liquidity_net: String,
    price0: String,
    price1: String,
}

#[derive(Debug)]
pub struct TokenDayData {
    price_usd: String,
}

#[derive(Debug)]
pub struct Token {
    id: String,
    name: String,
    symbol: String,
    volume_usd: String,
    logo_uri: String,
    decimals: String,
    token_day_data: Vec<TokenDayData>,
    total_value_locked_usd: String,
    pool_count: u32,
}

#[derive(Debug)]
pub struct PoolDayData {
    date: u64,
    volume_usd: String,
    open: String,
    high: String,
    low: String,
    close: String,
}

#[derive(Debug)]
pub struct Pool {
    id: String,
    fee_tier: String,
    liquidity: String,
    tick: String,
    sqrt_price: String,
    token0_price: String,
    token1_price: String,
    fee_growth_global0_x128: String,
    fee_growth_global1_x128: String,
    token0: Token,
    token1: Token,
    total_value_locked_usd: String,
    pool_day_data: Vec<PoolDayData>,
}

#[derive(Debug)]
pub struct Position {
    id: String,
    tick_lower: TickLower,
    tick_upper: TickUpper,
    deposited_token0: String,
    deposited_token1: String,
    liquidity: String,
    transaction: Transaction,
    collected_fees_token0: String,
    collected_fees_token1: String,
    fee_growth_inside0_last_x128: String,
    fee_growth_inside1_last_x128: String,
}

#[derive(Debug)]
pub struct TickLower {
    tick_idx: String,
    fee_growth_outside0_x128: String,
    fee_growth_outside1_x128: String,
}

#[derive(Debug)]
pub struct TickUpper {
    tick_idx: String,
    fee_growth_outside0_x128: String,
    fee_growth_outside1_x128: String,
}

#[derive(Debug)]
pub struct Transaction {
    timestamp: String,
}

lazy_static! {
    pub static ref Q96: BigUint = BigUint::from(2u32).pow(96);
    pub static ref Q128: BigUint = BigUint::from(2u32).pow(128);
    pub static ref ZERO: BigUint = BigUint::zero();
}

pub fn calculate_position_fees(
    pool: &Pool,
    position: &Position,
    token0: &Option<Token>,
    token1: &Option<Token>,
) -> Result<(BigUint, BigUint), std::num::ParseIntError> {
    let tick_current = pool.tick.parse::<i32>()?;
    let tick_lower = position.tick_lower.tick_idx.parse::<i32>()?;
    let tick_upper = position.tick_upper.tick_idx.parse::<i32>()?;
    let liquidity = BigUint::from_str(&position.liquidity).unwrap();

    // Global fee growth per liquidity '𝑓𝑔' for both token 0 and token 1
    let fee_growth_global_0 = BigUint::from_str(&pool.fee_growth_global0_x128).unwrap();
    let fee_growth_global_1 = BigUint::from_str(&pool.fee_growth_global1_x128).unwrap();

    // Fee growth outside '𝑓𝑜' of our lower tick for both token 0 and token 1
    let tick_lower_fee_growth_outside_0 =
        BigUint::from_str(&position.tick_lower.fee_growth_outside0_x128).unwrap();
    let tick_lower_fee_growth_outside_1 =
        BigUint::from_str(&position.tick_lower.fee_growth_outside1_x128).unwrap();

    // Fee growth outside '𝑓𝑜' of our upper tick for both token 0 and token 1
    let tick_upper_fee_growth_outside_0 =
        BigUint::from_str(&position.tick_upper.fee_growth_outside0_x128).unwrap();
    let tick_upper_fee_growth_outside_1 =
        BigUint::from_str(&position.tick_upper.fee_growth_outside1_x128).unwrap();

    let mut tick_lower_fee_growth_below_0 = ZERO.clone();
    let mut tick_lower_fee_growth_below_1 = ZERO.clone();
    let mut tick_upper_fee_growth_above_0 = ZERO.clone();
    let mut tick_upper_fee_growth_above_1 = ZERO.clone();

    if tick_current >= tick_lower {
        tick_lower_fee_growth_below_0 = tick_lower_fee_growth_outside_0;
        tick_lower_fee_growth_below_1 = tick_lower_fee_growth_outside_1;
    } else {
        tick_lower_fee_growth_below_0 =
            fee_growth_global_0.clone() - tick_lower_fee_growth_outside_0;
        tick_lower_fee_growth_below_1 =
            fee_growth_global_1.clone() - tick_lower_fee_growth_outside_1;
    }

    // These are the calculations for '𝑓𝑎(𝑖)' from the formula
    // for both token 0 and token 1
    if tick_current < tick_upper {
        tick_upper_fee_growth_above_0 = tick_upper_fee_growth_outside_0;
        tick_upper_fee_growth_above_1 = tick_upper_fee_growth_outside_1;
    } else {
        tick_upper_fee_growth_above_0 =
            fee_growth_global_0.clone() - tick_upper_fee_growth_outside_0;
        tick_upper_fee_growth_above_1 =
            fee_growth_global_1.clone() - tick_upper_fee_growth_outside_1;
    }

    // Calculations for '𝑓𝑟(𝑡1)' part of the '𝑓𝑢 =𝑙·(𝑓𝑟(𝑡1)−𝑓𝑟(𝑡0))' formula
    // for both token 0 and token 1
    let fr_t1_0 =
        fee_growth_global_0 - tick_lower_fee_growth_below_0 - tick_upper_fee_growth_above_0;
    let fr_t1_1 =
        fee_growth_global_1 - tick_lower_fee_growth_below_1 - tick_upper_fee_growth_above_1;

    // '𝑓𝑟(𝑡0)' part of the '𝑓𝑢 =𝑙·(𝑓𝑟(𝑡1)−𝑓𝑟(𝑡0))' formula
    // for both token 0 and token 1
    let fee_growth_inside_last_0 =
        BigUint::from_str(&position.fee_growth_inside0_last_x128).unwrap();
    let fee_growth_inside_last_1 =
        BigUint::from_str(&position.fee_growth_inside1_last_x128).unwrap();

    // The final calculations for the '𝑓𝑢 =𝑙·(𝑓𝑟(𝑡1)−𝑓𝑟(𝑡0))' uncollected fees formula
    // for both token 0 and token 1 since we now know everything that is needed to compute it

    let uncollected_fees_0 = mul_div(
        liquidity.clone(),
        fr_t1_0 - fee_growth_inside_last_0,
        (*Q128).clone(),
    );
    let uncollected_fees_1 = mul_div(
        liquidity,
        fr_t1_1 - fee_growth_inside_last_1,
        (*Q128).clone(),
    );

    // Decimal adjustment to get final results
    let uncollected_fees_adjusted_0 = uncollected_fees_0
        / expand_decimals(
            BigUint::from(1u32),
            token0
                .as_ref()
                .map_or(18, |t| t.decimals.parse().unwrap_or(18)),
        );
    let uncollected_fees_adjusted_1 = uncollected_fees_1
        / expand_decimals(
            BigUint::from(1u32),
            token1
                .as_ref()
                .map_or(18, |t| t.decimals.parse().unwrap_or(18)),
        );

    Ok((uncollected_fees_adjusted_0, uncollected_fees_adjusted_1))
}

fn encode_sqrt_price_x96(price: BigUint) -> BigUint {
    // let q96: num_bigint::BigUint = BigUint::from(1u64) << 96;
    let result = (price.to_f64().unwrap() * Q96.to_f64().unwrap()).sqrt();
    let result_str = format!("{:.0}", result); // Convert float to string with no decimal part
    BigUint::from_str(&result_str).unwrap()
}
// Helper function to expand decimals
fn expand_decimals(n: BigUint, exp: u32) -> BigUint {
    n * BigUint::from(10u32).pow(exp)
}

// Helper function to multiply and divide
fn mul_div(a: BigUint, b: BigUint, multiplier: BigUint) -> BigUint {
    a * b / multiplier
}
pub struct SimulationResult {
    date: i64,
    reserves_token1: f64,
    reserves_token2: f64,
    accumulated_fees: f64,
}

impl std::fmt::Debug for SimulationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimulationResult")
            .field("date", &self.date)
            .field("reserves_token1", &self.reserves_token1)
            .field("reserves_token2", &self.reserves_token2)
            .field("accumulated_fees", &self.accumulated_fees)
            .finish()
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
    let response = client
        .post(subgraph_url)
        .json(&GetPoolHourData::build_query(variables))
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
            match serde_json::from_str::<graphql_client::Response<get_pool_hour_data::ResponseData>>(
                &text_resp,
            ) {
                Ok(data) => {
                    if let Some(pool_hour_datas) =
                        data.data.and_then(|data| Some(data.pool_hour_datas))
                    {
                        pool_hour_datas
                    } else {
                        vec![]
                    }
                }
                Err(err) => {
                    println!("Failed to deserialize response: {:?}", err);
                    vec![]
                }
            }
        }
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
        // Convert tick to f64
        let tick_as_f64 = entry
            .tick
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        println!("tick_as_f64: {}", tick_as_f64); // Debug print

        if let (Some(lower), Some(upper)) = (lower_tick, upper_tick) {
            println!("lower: {}, upper: {}", lower, upper); // Debug print

            if tick_as_f64 >= lower && tick_as_f64 <= upper {
                // Parse fees_usd as f64 and multiply with fee_tier
                if let Ok(fees_usd_as_f64) = entry.fees_usd.parse::<f64>() {
                    accumulated_fees += fees_usd_as_f64 * fee_tier;
                }

                // Convert sqrt_price and volume_usd to f64
                let sqrt_price_as_f64 = entry.sqrt_price.parse::<f64>().unwrap_or(0.0);
                let volume_usd_as_f64 = entry.volume_usd.parse::<f64>().unwrap_or(0.0);
                reserves_token1 += volume_usd_as_f64 / sqrt_price_as_f64;
                reserves_token2 += volume_usd_as_f64 * sqrt_price_as_f64;

                results.push(SimulationResult {
                    date: entry.period_start_unix,
                    reserves_token1,
                    reserves_token2,
                    accumulated_fees,
                });
            }
        }
    }
    print!("{:?}", results);
    results
}
