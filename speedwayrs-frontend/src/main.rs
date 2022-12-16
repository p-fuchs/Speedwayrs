mod navbar;
mod account;
mod login;

pub use account::Account;
use dioxus::prelude::*;

#[derive(PartialEq, Props, Default)]
struct App {
    account_state: Account
}

fn start_application(cx: Scope) -> Element {
    let account_state = use_state(&cx, || Account::default());

    cx.render(rsx!(
        Router {
            navbar::render_navbar()

            Route {
                to: "/login",
                login::login_page {}
            }

            Route {
                to: "",
                "PAGE DOES NOT EXIST"
            }
        }
    ))
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::web::launch(start_application)
}
