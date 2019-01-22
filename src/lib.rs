#[macro_use]
extern crate serde_derive;

pub use self::session::{Session};
pub use self::scheme::{Config, Step, Product, Devices};

mod session;
mod scheme;


struct Token {
    start_token: (&'static str, i64),
    end_token: (&'static str, i64),
}

impl Token {
    fn new(start_token: &'static str, end_token: &'static str) -> Self {
        Token { start_token: (start_token, 0), end_token: (end_token, 0) }
    }

    fn set_offset(&mut self, start: i64, end: i64) {
        self.start_token.1 = start;
        self.end_token.1 = end;
    }

    fn parse(&self, html: &str) -> Option<(usize, usize)> {
        let pos = (html.find(self.start_token.0)? as i64 + self.start_token.1) as usize;
        let start = pos + self.start_token.0.len();
        let end = (html[start..].find(self.end_token.0)? as i64 + self.end_token.1) as usize;
        Some((start, start+end))
    }
}

fn remove_special_chars(text: &str) -> String {
    text
        .chars()
        .filter(|c| *c != '\n' && *c != ' ')
        .collect::<String>()
}

pub fn map_iccid_html(html: &str) -> std::collections::HashMap<String, String> {
    let without_special_chars = remove_special_chars(&html);
    let mut html = without_special_chars.as_str();

    let iccid_token = Token::new("ICCID:", r"</");
    let id_token = Token::new(r#"name="product"value=""#, r#"""#);

    let mut result = std::collections::HashMap::new();
    while let (Some((iccid_start, iccid_end)), Some((id_start, id_end))) = (iccid_token.parse(html), id_token.parse(html)) {
        let product_id = html[id_start..id_end].to_owned().to_string();
        let iccid = html[iccid_start..iccid_end].to_owned().to_string();

        result.insert(iccid, product_id);
        html = &html[iccid_end..];
    }
    result.to_owned()
}

pub fn parse_device_html(html: &str) -> Result<String, &'static str> {
    let trimmed_html = remove_special_chars(&html);
    let mut device_token = Token::new("sliderData=", "};");
    device_token.set_offset(0, 1);

    if let Some((start, end)) = device_token.parse(&trimmed_html) {
        Ok(trimmed_html[start..end].to_string())
    } else {
        Err("Data representation is changed.")
    }
}

pub fn login(session: &mut Session, config: &Config) -> reqwest::Result<reqwest::Response> {
    let params =  [
        ("IDToken1", config.name.as_str()),
        ("IDToken2", config.password.as_str()),
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

    fn yota_html(body: &str) -> String {
        format!(r#"
        <html>
            <head></head>
            <script>
                {}
                // some js logic
            </script>
        </html>
        "#, body)
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
        let devices_html = yota_html(format!("var sliderData = {};", data).as_str());

        let result = super::parse_device_html(&devices_html).unwrap();
        assert_eq!(result, super::remove_special_chars(&data));
    }

        #[test]
    fn test_parse_iccid_map() {
        let data = [("id123312", "iccid312123"), ("idtest", "iccidtest")];
        let html_body = data
            .iter()
            .map(|(k, v)| format!(
            r#"<form action="/selfcare/devices/changeOffer" method="post">
                <input type="hidden" name="product" value="{}" />
            </form>
            <span class="mac">
                ICCID:{}
            </span>"#, k, v))
            .fold("".to_string(), |mut acc, p| {
                acc.push_str(&p);
                acc.push('\n');
                acc
            });


        let result = super::map_iccid_html(&yota_html(&html_body));

        assert_eq!(result.iter().count(), 2);
        for (i, (k, v)) in result.iter().enumerate() {
            let (id, iccid) = data[i];
            assert_eq!(k, iccid);
            assert_eq!(v, id);
        }
    }
}
