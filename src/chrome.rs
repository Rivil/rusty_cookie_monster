use crate::cookie_values;
use directories::UserDirs;
use gethostname::gethostname;
use rusqlite::{Connection, Result};
use std::env;
use std::path::Path;

pub fn run_chrome_cookie() -> bool {
    let db_path = get_chrome_path();
    if db_path.is_none() {
        println!("Chrome path not found");
        return false;
    }
    match handle_cookie(&db_path.unwrap()) {
        Ok(_) => true,
        Err(e) => {
            println!("Error: {:?}", e);
            false
        }
    }
}

struct Cookie {
    creation_utc: i64,
    host_key: String,
    top_frame_site: String,
    name: String,
    value: String,
    encrypted_value: String,
    path: String,
    expires_utc: i64,
    is_secure: i64,
    is_httponly: i64,
    last_access_utc: i64,
    has_expires: i64,
    is_persistent: i64,
    priority: i64,
    samesite: i64,
    source_scheme: i64,
    source_port: i64,
    is_same_party: i64,
    last_update_utc: i64,
}

fn handle_cookie(db_path: &str) -> Result<bool> {
    let connection = Connection::open(db_path).unwrap();
    let cookie_values = cookie_values::CookieValue::get_values();

    let cookie = Cookie {
        creation_utc: cookie_values.creation_time.timestamp_micros() + 11644473600000000,
        host_key: cookie_values.host,
        name: cookie_values.name,
        value: gethostname().into_string().unwrap(),
        path: cookie_values.path,
        expires_utc: cookie_values.expiry.timestamp_micros() + 11644473600000000,
        is_secure: cookie_values.is_secure.into(),
        is_httponly: 0,
        last_access_utc: 0,
        has_expires: 1,
        samesite: cookie_values.same_site.into(),
        top_frame_site: "".to_string(),
        encrypted_value: "".to_string(),
        is_persistent: 1,
        priority: 1,
        source_scheme: 0,
        source_port: 0,
        is_same_party: 1,
        last_update_utc: 0,
    };

    let mut statement =
        connection.prepare("SELECT * FROM cookies WHERE host_key = ?1 AND name = ?2")?;

    let rows = statement.query_map([&cookie.host_key, &cookie.name], |row| {
        Ok(Cookie {
            creation_utc: row.get(1)?,
            host_key: row.get(2)?,
            name: row.get(4)?,
            value: row.get(5)?,
            path: row.get(6)?,
            expires_utc: row.get(7)?,
            is_secure: row.get(8)?,
            is_httponly: row.get(9)?,
            last_access_utc: row.get(10)?,
            has_expires: row.get(11)?,
            samesite: row.get(14)?,
            top_frame_site: row.get(17)?,
            encrypted_value: row.get(15)?,
            is_persistent: row.get(12)?,
            priority: row.get(13)?,
            source_scheme: row.get(16)?,
            source_port: row.get(18)?,
            is_same_party: row.get(19)?,
            last_update_utc: row.get(20)?,
        })
    })?;

    if rows.count() == 0 {
        insert_cookie(&connection, cookie)
    } else {
        update_cookie(&connection, cookie)
    }
}

fn insert_cookie(connection: &rusqlite::Connection, cookie: Cookie) -> Result<bool> {
    let query = "INSERT INTO cookies 
        (
            creation_utc, 
            host_key, 
            top_frame_site_key,
            name, 
            value, 
            encrypted_value,
            path, 
            expires_utc, 
            is_secure, 
            is_httponly, 
            last_access_utc, 
            has_expires, 
            is_persistent, 
            priority, 
            samesite, 
            source_scheme, 
            source_port, 
            is_same_party, 
            last_update_utc 
        ) 
        VALUES (?1, ?2, ?12, ?3, ?4, ?13, ?5, ?6, ?7, ?8, ?9, ?10, 1, 1, ?11, 2, 0, 0, 0)";

    let result = connection.execute(
        query,
        [
            cookie.creation_utc.to_string(),
            cookie.host_key.to_string(),
            cookie.name.to_string(),
            cookie.value.to_string(),
            cookie.path.to_string(),
            cookie.expires_utc.to_string(),
            cookie.is_secure.to_string(),
            cookie.is_httponly.to_string(),
            cookie.last_access_utc.to_string(),
            cookie.has_expires.to_string(),
            cookie.samesite.to_string(),
            cookie.top_frame_site,
            cookie.encrypted_value,
        ],
    );
    if result.is_err() {
        return Err(result.err().unwrap());
    }

    return Ok(true);
}

fn update_cookie(connection: &rusqlite::Connection, cookie: Cookie) -> Result<bool> {
    let query =
        "UPDATE cookies SET value = ?1, expires_utc = ?2  WHERE host_key = ?3 AND name = ?4";
    let result = connection.execute(
        query,
        [
            cookie.value.to_string(),
            cookie.expires_utc.to_string(),
            cookie.host_key.to_string(),
            cookie.name.to_string(),
        ],
    );
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    return Ok(true);
}

fn get_chrome_path() -> Option<String> {
    let path: String;

    match env::consts::OS {
        "linux" => {
            let user_dirs = UserDirs::new().unwrap();
            let home_dir = user_dirs.home_dir().to_str().unwrap();
            path = format!("{}/.config/google-chrome/Default/Cookies", home_dir);
        }
        "windows" => {
            let user_dirs = UserDirs::new().unwrap();
            let home_dir = user_dirs.home_dir().to_str().unwrap();
            path = format!(
                "{}/AppData/Local/Google/Chrome/User Data/Default/Network/Cookies",
                home_dir
            );
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
