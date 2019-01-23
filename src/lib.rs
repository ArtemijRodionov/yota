#[macro_use]
extern crate serde_derive;

pub use self::session::{Session};
pub use self::scheme::{Config};
use self::scheme::{Step, Product, Devices};

mod session;
mod scheme;

pub struct Yota<'a> {
    session: &'a mut Session,
    config: Config,
}

impl<'a> Yota<'a> {
    pub fn new(session: &'a mut Session, config: Config) -> Self {
        Yota { session, config }
    }

    pub fn change_speed(&mut self, speed: &str) -> Result<(), Box<std::error::Error>> {
        // todo: Yota's authorization is very slow.
        // Store cookies at disk to avoid it
        let mut resp = self.login()?;

        let text = resp.text().map(|t| remove_special_chars(&t))?;
        let iccid_id_map = map_iccid_html(&text);
        let id = iccid_id_map
            .get(self.config.iccid.as_str())
            .ok_or(format!(
                "{} ICCID doesn't exist. Choose one of: {:?}",
                &self.config.iccid,
                iccid_id_map.keys()
            ))?;

        let device_data = parse_device_html(&text)?;
        let devices = Devices::from_str(&device_data)?;

        // todo: prettify error messages. Show something useful
        let product = devices.get_product(&id)
            .ok_or(format!("{} product doesn't exist.", &id))?;
        let step = product.get_step(&speed)
            .ok_or(format!("{} speed doesn't exist.", &speed))?;

        self.change_offer(&product, &step)?;
        Ok(())
    }

    fn login(&mut self) -> reqwest::Result<reqwest::Response> {
        let params =  [
            ("IDToken1", self.config.name.as_str()),
            ("IDToken2", self.config.password.as_str()),
            ("goto", "https://my.yota.ru:443/selfcare/loginSuccess"),
            ("gotoOnFail", "https://my.yota.ru:443/selfcare/loginError"),
            ("org", "customer"),
            ("ForceAuth", "true"),
            ("old-token", ""),
        ];

        self.session.execute(
            reqwest::Method::POST,
            "https://login.yota.ru/UI/Login",
            |b| b.form(&params)
        )
    }

    fn get_devices(&mut self) -> reqwest::Result<reqwest::Response> {
        self.session.execute(
            reqwest::Method::GET,
            "https://my.yota.ru/selfcare/devices",
            |b| b
        )
    }

    fn change_offer(&mut self, product: &Product, step: &Step) -> reqwest::Result<reqwest::Response> {
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

        self.session.execute(
            reqwest::Method::POST,
            "https://my.yota.ru/selfcare/devices/changeOffer",
            |b| b.form(&params)
        )
    }
}

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
    let is_space = |c| [' ', '\n', '\t', '\r']
            .iter()
            .fold(false, |acc, s| { acc || (*s == c) });

    text
        .chars()
        .filter(|c| !is_space(*c))
        .collect::<String>()
}

fn map_iccid_html(mut html: &str) -> std::collections::HashMap<&str, &str> {
    let iccid_token = Token::new("ICCID:", r"</");
    let id_token = Token::new(r#"name="product"value=""#, r#"""#);

    let mut result = std::collections::HashMap::new();
    while let (Some((iccid_start, iccid_end)), Some((id_start, id_end))) = (iccid_token.parse(html), id_token.parse(html)) {
        let iccid = &html[iccid_start..iccid_end];
        let id = &html[id_start..id_end];

        result.insert(iccid, id);
        html = &html[iccid_end..];
    }
    result.to_owned()
}

fn parse_device_html(html: &str) -> Result<&str, &'static str> {
    let mut device_token = Token::new("sliderData=", "};");
    device_token.set_offset(0, 1);

    if let Some((start, end)) = device_token.parse(&html) {
        Ok(&html[start..end])
    } else {
        Err("Data representation is changed.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let html = format!(r#"
        <html>
            <head></head>
            <script>
                {}
                // some js logic
            </script>
        </html>
        "#, body);
        remove_special_chars(&html).to_string()
    }

    #[test]
    fn test_deserealization() {
        let data = yota_devices();
        let devices = Devices::from_str(&data).unwrap();

        assert_eq!(devices.mapped.len(), 2);
        for p in devices.mapped.values() {
            assert_eq!(p.steps.len(), 3);
        }
    }

    #[test]
    fn test_parse_devices() {
        let data = yota_devices();
        let devices_html = yota_html(format!("var sliderData = {};", data).as_str());

        let result = parse_device_html(&devices_html).unwrap();
        assert_eq!(result, remove_special_chars(&data));
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

        let html = yota_html(&html_body);
        let result = map_iccid_html(&html);

        assert_eq!(result.iter().count(), 2);
        for (id, iccid) in data.iter() {
            assert_eq!(result.get(iccid).unwrap(), id);
        }
    }
}
