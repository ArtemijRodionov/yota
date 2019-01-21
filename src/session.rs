use reqwest::{StatusCode, header::{COOKIE, LOCATION, SET_COOKIE, HeaderMap, HeaderValue}};

pub struct Session {
    client: reqwest::Client,
    cookies: cookie::CookieJar,
}

impl Session {
    // reqwest 0.9.6 doesn't handle `set-cookie` header in case of a redirect,
    // so i have to do it myself.
    // see this for detail: https://github.com/seanmonstar/reqwest/issues/14#issuecomment-342833155

    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::RedirectPolicy::none())
            .build()
            .expect("Client failed to initialize");
        Session { client, cookies: cookie::CookieJar::new() }
    }

    pub fn execute<F, U>(
        &mut self, mut method: reqwest::Method, url: U, cb: F
    ) -> reqwest::Result<reqwest::Response>
        where F: FnOnce(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
              U: reqwest::IntoUrl
    {
        let url = url.into_url()?;

        let req = cb(self.client.request(method.clone(), url.clone()))
            .headers(self.cookie_header(&url))
            .build()?;
        let mut resp = self.client.execute(req)?;

        self.set_cookies(&url, &resp.headers());

        // loop until isn't redirection
        loop {
            let status = resp.status();
            if status.is_redirection() {
                let mut body = vec![];
                resp.copy_to(&mut body)?;

                let loc = resp
                    .headers()
                    .get(LOCATION)
                    .expect("Redirection without Location header.")
                    .to_str()
                    .expect("Ivalid characters in Location header.")
                    .parse::<reqwest::Url>()
                    .expect("Can't parse Location header.");

                let mut headers = resp.headers().clone();
                loop {
                    if let None = headers.remove(SET_COOKIE) {
                        break;
                    }
                }

                // keep referrer method for 307-308
                method = match status {
                    StatusCode::TEMPORARY_REDIRECT |
                    StatusCode::PERMANENT_REDIRECT => method,
                    _                              => reqwest::Method::GET
                };

                let req = self.client
                    .request(method.clone(), loc.clone())
                    .body(body)
                    .headers(headers)
                    .headers(self.cookie_header(&loc))
                    .build()?;

                resp = self.client.execute(req)?;
                self.set_cookies(&loc, &resp.headers());
            } else {
                break Ok(resp)
            }
        }
    }

    fn cookie_header(&self, url: &reqwest::Url) -> HeaderMap
    {
        let domain = url.domain().unwrap_or("");
        let path = url.path();

        let cookie_value = self.cookies.iter()
            .filter(|c| match c.domain() {
                Some(d) => domain == d || domain.ends_with(d),
                None    => false
            } && match c.path() {
                Some(p) => path.starts_with(p),
                None    => false
            } && match c.expires() {
                Some(t) => t > time::now(),
                None    => true
            })
            .fold("".to_string(), |a, c| {
                // todo: remove format to avoid excess allocations
                let (name, value) = c.name_value();
                if a.is_empty() {
                    format!("{}{}={}", a, name, value)
                } else {
                    format!("{}; {}={}", a, name, value)
                }
            });

        let mut header = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(cookie_value.as_str()) {
            header.insert(COOKIE, value);
        }
        header
    }

    fn set_cookies(&mut self, url: &reqwest::Url, headers: &HeaderMap)
    {
        let domain = url.domain().unwrap_or("");
        let path = url.path();

        headers.get_all(SET_COOKIE).iter()
            .filter_map(|sc| sc.to_str().ok())
            .filter_map(|s| s.parse::<cookie::Cookie>().ok())
            // throw away a cookie with empty Domain if url's domain is empty too
            .filter(|c| !(domain.is_empty() && c.domain().is_none()))
            .for_each(|mut c| {
                if let None = c.domain() {
                    c.set_domain(domain.to_owned());
                }
                if let None = c.path() {
                    c.set_path(path.to_owned());
                }
                self.cookies.add(c);
            });
    }
}