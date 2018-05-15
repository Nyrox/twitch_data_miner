extern crate curl;
extern crate serde_json;

use curl::easy::{Easy, List};


pub struct TwitchAPI {
    token: String,
    base_url: String
}

impl TwitchAPI {
    pub fn new(token: String) -> TwitchAPI {
        TwitchAPI { token, base_url: "https://api.twitch.tv/helix/".to_owned()  }
    }

    pub fn generic_request(&self, url: &str) -> serde_json::Value {
        let mut easy = Easy::new();
        let mut buf = Vec::new();

        easy.url(url);
        let mut headers = List::new();
        headers.append(&format!("Authorization: Bearer {}", self.token));
        easy.http_headers(headers);

        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                buf.extend_from_slice(data);
                Ok(data.len())
            }).expect("Transfer write failed.");
            transfer.perform().expect("Transfer failed");
        }

        return serde_json::from_str(&String::from_utf8(buf).expect("Request returned invalid utf8 string")).expect("Request returned invalid json");
    }

    pub fn request(&mut self) -> Request {
        Request::new(self)
    }
}


pub struct Request<'a> {
    api: &'a TwitchAPI,
    resource: String,
    params: Vec<(String, String)>,
}

impl<'a> Request<'a> {
    pub fn new(api: &'a TwitchAPI) -> Request<'a> {
        Request { api, resource: String::new(), params: Vec::new() }
    }

    pub fn resource(mut self, r: String) -> Request<'a> {
        self.resource = r;
        return self;
    }

    pub fn param(mut self, p: (String, String)) -> Request<'a> {
        self.params.push(p);
        return self;
    }

    pub fn get(&self) -> serde_json::Value {
        let mut queryString = String::new();
        for i in 0..self.params.len() {
            if i != 0 { queryString.push('&') };
            queryString.push_str(&format!("{}={}", self.params[i].0, self.params[i].1));
        }
        self.api.generic_request(&format!("{}/{}/?{}", self.api.base_url, self.resource, queryString))
    }
}
