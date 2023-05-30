pub struct CookieValue {
    pub host: String,
    pub name: String,
    pub path: String,
    pub is_secure: i8,
    pub same_site: i8,
}

impl CookieValue {
    pub fn get_values() -> CookieValue {
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
        }
    }
}
