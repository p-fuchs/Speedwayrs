use gloo_net::http::{
    ReferrerPolicy, Request, RequestCache, RequestCredentials, RequestMode, Response,
};
use once_cell::sync::OnceCell;
use std::fmt::Write;
use sycamore::{
    futures::spawn_local_scoped,
    reactive::{Scope, Signal},
    web::Html,
};

const SESSION_ADDRESS: &'static str = const_format::formatcp!("{}/session", crate::SERVER_ADDRESS);

pub fn get<'a, U: Into<&'a str>>(url: U) -> Request {
    Request::get(url.into())
}

pub fn post<'a, U: Into<&'a str>>(url: U) -> Request {
    Request::post(url.into())
}

pub async fn execute(mut request: Request) -> Result<Response, gloo_net::Error> {
    request = request
        .referrer_policy(ReferrerPolicy::UnsafeUrl)
        .mode(RequestMode::Cors)
        .credentials(RequestCredentials::Include)
        .cache(RequestCache::Default);

    request.send().await
}

pub async fn update_session_info<'a>(cx: Scope<'a>, username_data: &'a Signal<Option<String>>) {
    match session_info().await {
        Ok(possible_user) => {
            log::debug!("Update user {:?}", possible_user);
            username_data.set(possible_user);
        }
        Err(e) => {
            log::error!("Error while getting session info. {:?}", e);
        }
    }

    log::debug!("Session info: {:?}", username_data.get().as_ref());
}

async fn session_info() -> Result<Option<String>, gloo_net::Error> {
    let request = gloo_net::http::Request::get(SESSION_ADDRESS);

    let response = execute(request).await?;
    log::debug!("Got session info: {:?}", response);

    match response.text().await {
        Err(e) => {
            log::error!("Cannot get session response payload: {e:?}");

            Ok(None)
        }
        Ok(body) => {
            log::debug!("Response body: {body:?}");

            if body == "" {
                Ok(None)
            } else {
                Ok(Some(body))
            }
        }
    }
}
/*

pub async fn execute(mut request: reqwest::RequestBuilder) -> Result<reqwest::Response, reqwest::Error> {
    static JAR_CELL: OnceCell<RwLock<CookieJar>> = OnceCell::new();

    let jar = JAR_CELL.get_or_init(|| {
        RwLock::new(CookieJar::new())
    });

    {
        let jar_read = jar.read().await;
        log::error!("COOKIES: {:?}", jar_read);

        let mut cookies = String::new();

        let mut cookie_iter = jar_read.iter();
        if let Some(cookie) = cookie_iter.next() {
            write!(&mut cookies, "{}", cookie.stripped()).unwrap();
        }

        for c in cookie_iter {
            write!(&mut cookies, ";{}", c.stripped()).unwrap();
        }

        let cookies = HeaderValue::from_str(&cookies).unwrap();
        request = request.header(reqwest::header::COOKIE, cookies);
    }

    let response = request.send().await?;
    // log::debug!("RESPONSE: {:?}", response.text().await);
    let headers = response.headers();

    let mut cookies = Vec::new();

    for set_cookie in headers.get_all(reqwest::header::SET_COOKIE) {
        log::debug!("COOKIE SET: {set_cookie:?}");
        let cookie = Cookie::parse_encoded(set_cookie.to_str().unwrap().to_string());

        if let Ok(cookie) = cookie {
            cookies.push(cookie);
        }
    }

    let mut jar_write = jar.write().await;
    log::debug!("COOKIES: {}", cookies.len());

    for cookie in cookies {
        jar_write.add(cookie);
    }

    Ok(response)

    */
