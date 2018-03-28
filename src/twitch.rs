extern crate curl;
extern crate serde_json;

use curl::easy::{Easy, List};


pub struct TwitchAPI {
    token: String
}

impl TwitchAPI {
    pub fn new(token: String) -> TwitchAPI {
        TwitchAPI { token }
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
}
