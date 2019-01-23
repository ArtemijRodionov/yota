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

        self.drop_cookies();
        let req = cb(self.client.request(method.clone(), url.clone()))
            .headers(self.cookie_header(&url))
            .build()?;
        let mut resp = self.client.execute(req)?;

        self.set_cookies(&url, &resp.headers());

        // loop until isn't redirection
        loop {
            let status = resp.status();
            if !status.is_redirection() {
                break Ok(resp)
            }

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
            while let Some(_) = headers.remove(SET_COOKIE) {}

            // keep referrer method for 307-308
            method = match status {
                StatusCode::TEMPORARY_REDIRECT |
                StatusCode::PERMANENT_REDIRECT => method,
                _                              => reqwest::Method::GET
            };

            self.drop_cookies();
            let req = self.client
                .request(method.clone(), loc.clone())
                .body(body)
                .headers(headers)
                .headers(self.cookie_header(&loc))
                .build()?;

            resp = self.client.execute(req)?;
            self.set_cookies(&loc, &resp.headers());
        }
    }

    fn drop_cookies(&mut self) {
        let to_remove: Vec<cookie::Cookie> = self.cookies
            .iter()
            // max-age is not implemented
            .filter(|c| match c.expires() {
                Some(t) => t < time::now(),
                None    => false
            })
            .map(|c| c.clone())
            .collect();

        to_remove.into_iter().for_each(|c| self.cookies.force_remove(c));
    }

    /// Gives matched cookies for a request.
    ///
    /// Maps cookies from cookie-rs into hyper, filters them
    /// by the url's domain and path.
    fn cookie_header(&self, url: &reqwest::Url) -> HeaderMap
    {
        let domain = url.domain().unwrap_or("");
        let path = url.path();

        let cookie_value = self.cookies.iter()
            .filter(|c| match c.domain() {
                // it doesn't handle a cookie with
                // top level domain and some subdomains: ".com", ".co.uk" ...
                // so you SHOULD NOT use it anywhere if you are working with multiple domains
                Some(d) => domain == d || domain.ends_with(d),
                None    => false
            } && match c.path() {
                Some(p) => path.starts_with(p),
                None    => false
            })
            .fold("".to_string(), |mut a, c| {
                let (name, value) = c.name_value();
                if !a.is_empty() {
                    a.push_str("; ");
                }
                a.push_str(name);
                a.push_str("=");
                a.push_str(value);
                a
            });

        let mut header = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(cookie_value.as_str()) {
            header.insert(COOKIE, value);
        }
        header
    }

    /// Saves cookies to the session.
    ///
    /// Maps cookies from hyper into cookie-rs and stores them in CookieJar.
    fn set_cookies(&mut self, url: &reqwest::Url, headers: &HeaderMap)
    {
        let domain = url.domain().unwrap_or("");
        let path = url.path();

        headers.get_all(SET_COOKIE).iter()
            .filter_map(|sc| sc.to_str().ok())
            .filter_map(|s| s.parse::<cookie::Cookie>().ok())
            .filter(|c| if let Some(c_domain) = c.domain() {
                // it doesn't handle a cookie with
                // top level domain and some subdomains: ".com", ".co.uk" ...
                // so you SHOULD NOT use it if you are working with multiple domains
                domain == c_domain || domain.ends_with(c_domain)
            } else {
                !domain.is_empty()
            })
            .for_each(|mut c| {
                if let None = c.domain() {
                    c.set_domain(domain.to_owned());
                }
                if let None = c.path() {
                    c.set_path(path.to_owned());
                }
                self.cookies.add_original(c);
            });
    }
}