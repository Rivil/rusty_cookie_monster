use crate::cookie_values;
use directories::UserDirs;
use gethostname::gethostname;
use rusqlite::{Connection, Result};
use std::env;
use std::path::Path;

pub fn run_firefox_cookie() -> bool {
    let firefox_path = get_firefox_path();

    if firefox_path.is_none() {
        println!("Firefox path not found");
        return false;
    }
    let db_path = firefox_get_database_path(&firefox_path.unwrap());

    match handle_cookie(&db_path) {
        Ok(_) => true,
        Err(e) => {
            println!("Error: {:?}", e);
            false
        }
    }
}

struct Cookie {
    host: String,
    name: String,
    value: String,
    path: String,
    expiry: i64,
    last_accessed: i64,
    creation_time: i64,
    is_secure: i64,
    is_http_only: i64,
    in_browser_element: i64,
    same_site: i64,
    raw_same_site: i64,
    scheme_map: i64,
}

fn handle_cookie(db_path: &str) -> Result<bool> {
    let connection = Connection::open(db_path).unwrap();
    let cookie_values = cookie_values::CookieValue::get_values();

    let cookie = Cookie {
        host: cookie_values.host,
        name: cookie_values.name,
        value: gethostname().into_string().unwrap(),
        path: cookie_values.path,
        expiry: cookie_values.expiry.timestamp(),
        last_accessed: 0,
        creation_time: cookie_values.creation_time.timestamp_micros(),
        is_secure: cookie_values.is_secure.into(),
        is_http_only: 0,
        in_browser_element: 0,
        same_site: cookie_values.same_site.into(),
        raw_same_site: 0,
        scheme_map: 0,
    };

    let mut statement =
        connection.prepare("SELECT * FROM moz_cookies WHERE host = ?1 AND name = ?2")?;

    let rows = statement.query_map([&cookie.host, &cookie.name], |row| {
        Ok(Cookie {
            host: row.get(4)?,
            name: row.get(2)?,
            value: row.get(3)?,
            path: row.get(5)?,
            expiry: row.get(6)?,
            last_accessed: row.get(7)?,
            creation_time: row.get(8)?,
            is_secure: row.get(9)?,
            is_http_only: row.get(10)?,
            in_browser_element: row.get(11)?,
            same_site: row.get(12)?,
            raw_same_site: row.get(13)?,
            scheme_map: row.get(14)?,
        })
    })?;
    if rows.count() == 0 {
        return insert(&connection, cookie);
    } else {
        return update(&connection, cookie);
    }
}
fn insert(connection: &rusqlite::Connection, cookie: Cookie) -> Result<bool> {
    let query = "INSERT INTO moz_cookies (host, name, value, path, expiry, lastAccessed, creationTime, isSecure, isHttpOnly, inBrowserElement, sameSite, rawSameSite, schemeMap) 
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)";

    let result = connection.execute(
        query,
        [
            cookie.host,
            cookie.name,
            cookie.value,
            cookie.path,
            cookie.expiry.to_string(),
            cookie.last_accessed.to_string(),
            cookie.creation_time.to_string(),
            cookie.is_secure.to_string(),
            cookie.is_http_only.to_string(),
            cookie.in_browser_element.to_string(),
            cookie.same_site.to_string(),
            cookie.raw_same_site.to_string(),
            cookie.scheme_map.to_string(),
        ],
    );

    if result.is_err() {
        return Err(result.unwrap_err());
    }

    return Ok(true);
}

fn update(connection: &rusqlite::Connection, cookie: Cookie) -> Result<bool> {
    let query = "UPDATE moz_cookies SET expiry = {}, value = {} WHERE host = {} AND name = {}";
    let result = connection.execute(
        query,
        [
            cookie.expiry.to_string(),
            cookie.value,
            cookie.host,
            cookie.name,
        ],
    );

    if result.is_err() {
        return Err(result.unwrap_err());
    }
    return Ok(true);
}

fn get_firefox_path() -> Option<String> {
    let path: String;

    match env::consts::OS {
        "linux" => {
            let user_dirs = UserDirs::new().unwrap();
            let home_dir = user_dirs.home_dir().to_str().unwrap();
            path = format!("{}/.mozilla/firefox/", home_dir);
        }
        "windows" => {
            let user_dirs = UserDirs::new().unwrap();
            let home_dir = user_dirs.home_dir().to_str().unwrap();
            path = format!("{}/AppData/Roaming/Mozilla/Firefox/", home_dir);
        }
        _ => {
            println!("OS not supported");
            return None;
        }
    }
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

fn firefox_get_database_path(path: &str) -> String {
    let conf = ini!(format!("{}profiles.ini", path).as_str());
    let mut return_path = String::new();

    for (sec, prop) in conf.iter() {
        for (key, val) in prop.iter() {
            if key == "name" && val.as_ref().unwrap() == "default-release" {
                return_path = format!(
                    "{}{}/cookies.sqlite",
                    path,
                    conf[sec]["path"].clone().unwrap()
                );
                break;
            }
        }
    }
    return_path
}
