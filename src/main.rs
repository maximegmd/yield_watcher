use std::fs::OpenOptions;
use chrono::Utc;
use tokio::time::Duration;
use serde::{Deserialize, Serialize};
use chrono::DateTime;
use simple_logger::SimpleLogger;
use log::{info, error};
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(help = "The public BSC address you want to track")]
    bsc_address: String,
}

mod belt;

#[derive(Deserialize, Serialize, Debug, Default)]
struct FarmDetails {
    id: String,
    name: String,
    provider: String
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct FarmValue {
    symbol: String,
    amount: f64,
    usd: Option<f64>
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Farm {
    farm: FarmDetails,
    rewards: Vec<FarmValue>,
    deposit: FarmValue,
    icon: String,
    farm_rewards: f64
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct Platform {
    id: String,
    label: String,
    url: String,
    icon: String,
    token: String,
    token_price: f64,
    name: String,
    farms: Vec<Farm>
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct FarmResponse {
    platforms: Vec<Platform>
}

#[derive(Serialize)]
struct Row<'a> {
    farm_id: &'a str,
    date: &'a str,
    token_deposit: f64,
    usd_value: f64,
    usd_reward: f64
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let args = Cli::from_args();

    let req = "https://farm.army/api/v0/farms/".to_owned() + &args.bsc_address;

    let file_name = "farm_yield.csv";
    let add_headers = !Path::new(file_name).exists();

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(file_name)
        .unwrap();

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(add_headers)
        .from_writer(file);

    loop{

        let now: DateTime<Utc> = Utc::now();
        let belt_price = belt::get_price().await.unwrap_or(1 as f64);

        let req_result = reqwest::get(&req).await;
        if let Ok(res) = req_result {
            let json_result = res.json::<FarmResponse>().await;
            if let Ok(json) = json_result {

                let platforms = &json.platforms;
                for platform in platforms {
                    let farms = &platform.farms;
                    for farm in farms {
                        let farm_id = farm.farm.id.clone();
                        let token_value = farm.deposit.amount.clone();
                        let mut usd_value = farm.deposit.usd.clone();
                        let total_usd_reward = farm.farm_rewards.clone();
                        let date = now.to_rfc3339();

                        if farm.farm.name == "4BELT" {
                            usd_value = Some(token_value * belt_price);
                        }

                        let row = Row {
                            farm_id: &farm_id,
                            date: &date,
                            token_deposit: token_value,
                            usd_value: usd_value.unwrap_or(0 as f64),
                            usd_reward: total_usd_reward
                        };

                        if let Err(err) = wtr.serialize(row) {
                            error!("Error serializing record {:?}", err);
                        }
                    }
                }

                info!("Serialized at {}", now.to_rfc3339());

                if let Err(err) = wtr.flush(){
                    error!("Error flushing {:?}", err);
                }
            } else if let Err(res) = json_result {
                error!("Json parse failed with {:?}", res);
            }
        }
        else if let Err(res) = req_result {
            error!("Request failed with {:?}", res);
        }

        tokio::time::sleep(Duration::from_secs(60 * 60)).await;
    }
}