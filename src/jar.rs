use std::env;

use cookie::{time::Duration, Cookie, CookieJar, Key};
use entity::users::Model as User;

pub fn get_auth_cookie(user: &User) -> String {
    let master_key = env::var("COOKIE_KEY").expect("env variable COOKIE_KEY not set");
    let key = Key::derive_from(master_key.as_bytes());
    let mut jar = CookieJar::new();
    let mut priv_jar = jar.private_mut(&key);

    let cookie = Cookie::build("recipes_auth", user.id.to_string())
        .path("/")
        .same_site(cookie::SameSite::None)
        .secure(true)
        .http_only(true)
        .max_age(Duration::days(30))
        .finish();

    priv_jar.add(cookie);

    jar.get("recipes_auth").unwrap().to_string()
}
