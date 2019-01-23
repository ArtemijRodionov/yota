use std::io::{Read};

use serde::de::{Deserialize};
use serde_json::Number;

fn number_to_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let n: Number = Deserialize::deserialize(d)?;
    Ok(n.to_string())
}

#[derive(Deserialize)]
pub struct Config {
    pub name: String,
    pub password: String,
    pub iccid: String,
}

impl Config {
    pub fn open(path: &str) -> Result<Self, Box<std::error::Error>> {
        let mut file = std::fs::File::open(path)?;
        let mut config_data = String::new();
        file.read_to_string(&mut config_data)?;
        Ok(serde_json::from_str::<Self>(&config_data)?)
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Step
{
    pub code: String,
    speed_number: String,
    // todo: show this fields in `yota status`
    // speed_string: String,
    // amount_number: String,
    // amount_string: String,
    // remain_number: String,
    // remain_string: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize, Debug)]
pub struct Product {
    #[serde(deserialize_with = "number_to_string")]
    pub product_id: String,
    pub steps: Vec<Step>,
}

impl Product {
    pub fn get_step(&self, speed: &str) -> Option<&Step> {
        self.steps
            .iter()
            .skip_while(|s| { s.speed_number != *speed })
            .next()
    }
}

#[derive(Deserialize, Debug)]
pub struct Devices {
    #[serde(flatten)]
    pub mapped: std::collections::HashMap<String, Product>
}

impl Devices {
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
       serde_json::from_str::<Self>(s)
    }

    pub fn get_product(&self, product: &str) -> Option<&Product> {
        self.mapped.get(product)
    }
}
