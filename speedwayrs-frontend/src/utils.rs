use gloo_net::{http::Response, Error};
use serde::{de::DeserializeOwned, Serialize};

pub async fn fetch_response<S: Serialize>(
    source: &str,
    body: &S
) -> Result<Response, Error> {
    let json_body = serde_json::to_string(body).unwrap();

    let request = gloo_net::http::Request::post(source)
        .header("Content-Type", "application/json")
        .body(&json_body);

    crate::client::execute(request).await
}

pub async fn fetch_get<T: DeserializeOwned>(
    source: &str
) -> Option<T> {
    let request = gloo_net::http::Request::get(source);

    match crate::client::execute(request).await {
        Err(e) => {
            log::error!("Error while querying get. Error = [{e:?}]");

            None
        }
        Ok(resp) => {
            if let Ok(body) = resp.text().await {
                serde_json::from_str(&body).ok()
            } else {
                None
            }
        }
    }
}

pub async fn fetch_json_data<T: DeserializeOwned, S: Serialize>(
    source: &str,
    body: &S,
) -> Option<T> {
    let json_body = match serde_json::to_string(body) {
        Ok(body) => body,
        Err(e) => {
            log::error!("Error while serializing in fetch_json_data. Error = [{e:?}]");

            return None;
        }
    };

    let request = gloo_net::http::Request::post(source)
        .header("Content-Type", "application/json")
        .body(&json_body);

    match crate::client::execute(request).await {
        Ok(response) => {
            if response.status() != http::StatusCode::OK {
                log::error!("Server returned (fetch_json_data): {}", response.status());
                return None;
            }

            match response.text().await {
                Ok(body_text) => serde_json::from_str(&body_text).ok(),
                Err(e) => {
                    log::error!("Cannot fetch response text body. Error = [{e:?}]");

                    None
                }
            }
        }
        Err(e) => {
            log::error!("Server returned error. Error = [{e:?}]");

            None
        }
    }
}
