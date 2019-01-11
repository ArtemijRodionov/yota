#[macro_use]
extern crate serde_derive;

pub use self::session::{Session};
pub use self::scheme::{Config, Step, Product, Devices};

mod session;
mod scheme;

pub fn parse_device_html(device_html: &str) -> Result<String, &'static str> {
    let trimmed_html = device_html.replace("\n", "");
    let data_keyword = "sliderData = {";

    if let Some(start) = trimmed_html.find(data_keyword) {
        let data_start = &trimmed_html[(start+data_keyword.len()-1)..];
        let end = data_start.find("};").unwrap();
        Ok(data_start[..end+1].to_string())
    } else {
        Err("Data representation is changed.")
    }
}

pub fn login(session: &mut Session, name: &str, pass: &str) -> reqwest::Result<reqwest::Response> {
    let params =  [
        ("IDToken1", name),
        ("IDToken2", pass),
        ("goto", "https://my.yota.ru:443/selfcare/loginSuccess"),
        ("gotoOnFail", "https://my.yota.ru:443/selfcare/loginError"),
        ("org", "customer"),
        ("ForceAuth", "true"),
        ("old-token", ""),
    ];

    session.execute(
        reqwest::Method::POST,
        "https://login.yota.ru/UI/Login",
        |b| b.form(&params)
    )
}

pub fn get_devices(session: &mut Session) -> reqwest::Result<reqwest::Response> {
    session.execute(
        reqwest::Method::GET,
        "https://my.yota.ru/selfcare/devices",
        |b| b
    )
}

pub fn change_offer(session: &mut Session, product: &Product, step: &Step) -> reqwest::Result<reqwest::Response> {
    let params =  [
        ("product", product.product_id.as_str()),
        ("offerCode", step.code.as_str()),
        ("areOffersAvailable", "false"),
        ("status", "custom"),
        ("autoprolong", "0"),
        ("isSlot", "false"),
        ("currentDevice", "1"),
        ("isDisablingAutoprolong", "false"),
        ("resourceId", ""),
        ("username", ""),
        ("homeOfferCode", ""),
        ("period", ""),
        ("homeOfferCode", ""),
    ];

    session.execute(
        reqwest::Method::POST,
        "https://my.yota.ru/selfcare/devices/changeOffer",
        |b| b.form(&params)
    )
}

#[cfg(test)]
mod tests {
    fn yota_devices() -> String {
        let offer_code = "POS-MA6-0005";
        let step = format!(r#"{{
            "code": "{}",
            "amountNumber": "450",
            "amountString": "руб. в месяц",
            "remainNumber": "56",
            "remainString": "дней",
            "returnAmount": "0",
            "speedNumber": "5.0",
            "speedString": "Кбит/сек (макс.)",
            "description": "&nbsp;"
        }}"#, offer_code);
        let product = format!(r#"{{
            "offerCode": "{}",
            "productId": 123321123,
            "steps": [{1}, {1}, {1}],
            "status": "custom"
        }}"#, offer_code, step);
        format!(r#"{{ "123312123": {0}, "312123321": {0} }}"#, product)
    }


    #[test]
    fn test_deserealization() {
        let data = yota_devices();
        let result = super::Devices::from_str(&data).unwrap();

        assert_eq!(result.mapped.len(), 2);
        for p in result.mapped.values() {
            assert_eq!(p.steps.len(), 3);
        }
    }

    #[test]
    fn test_parse_devices() {
        let data = yota_devices();
        let devices_html = format!(r#"
        <html>
            <head></head>
            <script>
                var sliderData = {};
                // some js logic
            </script>
        </html>
        "#, data);

        let result = super::parse_device_html(&devices_html).unwrap();
        assert_eq!(result, data.replace("\n", ""));
    }
}
