use serde::de::{Deserialize};
use serde_json::Number;

fn number_to_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let n: Number = Deserialize::deserialize(d)?;
    Ok(n.to_string())
}


#[derive(Deserialize)]
pub struct Config {
    pub name: String,
    pub password: String
}

impl Config {
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
       serde_json::from_str::<Self>(s)
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize)]
pub struct Step
{
    pub code: String,
    amount_number: String,
    amount_string: String,
    remain_number: String,
    remain_string: String,
    speed_number: String,
    speed_string: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Deserialize)]
pub struct Product {
    #[serde(deserialize_with = "number_to_string")]
    pub product_id: String,
    pub steps: Vec<Step>,
}

impl Product {
    pub fn find_step(&self, speed: &String) -> Result<&Step, &'static str> {
        match self.steps
            .iter()
            .skip_while(|s| { s.speed_number != *speed })
            .next() {
                Some(step) => Ok(step),
                None       => Err("Step not found.")
            }
    }
}

#[derive(Deserialize)]
pub struct Devices {
    #[serde(flatten)]
    pub mapped: std::collections::HashMap<String, Product>
}

impl Devices {
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
       serde_json::from_str::<Self>(s)
    }

    pub fn find_product(&self, product_id: &String) -> Result<&Product, &'static str> {
        match self.mapped.get(product_id) {
            Some(product) => Ok(product),
            None          => Err("Product not found.")
        }
    }
}
