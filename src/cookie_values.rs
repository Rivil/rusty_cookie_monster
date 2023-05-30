use chrono::{DateTime, Duration, Utc};

pub struct CookieValue {
    pub host: String,
    pub name: String,
    pub path: String,
    pub is_secure: i8,
    pub same_site: i8,
    pub expiry: DateTime<Utc>,
    pub creation_time: DateTime<Utc>,
}

impl CookieValue {
    pub fn get_values() -> CookieValue {
        let now = Utc::now();
        let conf = ini!("Settings.ini");
        CookieValue {
            host: conf["general"]["host"].clone().unwrap(),
            name: conf["general"]["name"].clone().unwrap(),
            is_secure: conf["general"]["is_secure"]
                .clone()
                .unwrap()
                .parse::<i8>()
                .unwrap(),
            same_site: conf["general"]["same_site"]
                .clone()
                .unwrap()
                .parse::<i8>()
                .unwrap(),
            path: conf["general"]["path"].clone().unwrap(),
            expiry: (now.clone() + Duration::days(30)),
            creation_time: now,
        }
    }
}
