use serde::{Deserialize, Serialize};
use log::error;


#[derive(Deserialize, Serialize, Debug, Default)]
struct BeltEntry {
    name: String,
    ticker: String,
    address: String,
    img: String,
    price: String
}

pub async fn get_price() -> Result<f64, String> {
    let req_result = reqwest::get("https://s.belt.fi/status/A_beltTokenList.json").await;
    if let Ok(req_result) = req_result {
        let json_result = req_result.json::<Vec::<BeltEntry>>().await;
        if let Ok(json) = json_result {
            for entry in json {
                if entry.ticker == "4BELTPOOLLP" {
                    return Ok(entry.price.parse::<f64>().unwrap_or(1.0));
                }
            }
        } else if let Err(err) = json_result {
            error!("Request parsing json {:?}", err);
        }
    } else if let Err(err) = req_result {
        error!("Request failed with {:?}", err);
    }

    return Err("Can't retrieve 4belt price".to_owned());
}
