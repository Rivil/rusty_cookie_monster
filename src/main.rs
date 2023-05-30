mod cookie_values;
mod firefox;
#[macro_use]
extern crate ini;

fn main() {
    let firefox_result = firefox::run_firefox_cookie();
    println!("Firefox: {}", firefox_result);
}
