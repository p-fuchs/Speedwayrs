mod login;
mod navbar;
mod signup;
mod client;

use cookie::{Cookie, CookieJar};
use login::LoginPage;
use navbar::Navbar;
use once_cell::sync::OnceCell;
use signup::SignupPage;
use sycamore::{
    reactive::{create_signal, ReadSignal, Scope, Signal},
    view,
    view::View,
    web::Html,
    Prop, futures::spawn_local_scoped,
};
use sycamore_router::{HistoryIntegration, Route, Router};
use std::fmt::Write;

const SERVER_ADDRESS: &'static str = "http://localhost:47123";

#[derive(Prop, Clone, Copy)]
pub struct ApplicationData<'a> {
    username: &'a Signal<Option<String>>,
}

impl<'a> ApplicationData<'a> {
    pub fn get_username(&self) -> &'a Signal<Option<String>> {
        self.username
    }
}

#[derive(Route, Debug)]
pub enum ApplicationRoute {
    #[to("/")]
    Home,
    #[to("/login")]
    Login,
    #[to("/signup")]
    Signup,
    #[not_found]
    NotFound,
}

fn start_application<G: Html>(cx: Scope) -> View<G> {
    let username_data = create_signal(cx, None);

    spawn_local_scoped(cx, async move {
        client::update_session_info(cx, username_data).await;
    });

    view! {
        cx,
        Router(
            integration=HistoryIntegration::new(),
            view = move |cx, route: &ReadSignal<ApplicationRoute>| {
                view! { cx,
                    div() {
                        Navbar(username=username_data)

                        (
                            match route.get().as_ref() {
                                ApplicationRoute::Login => {
                                    view! {
                                        cx,
                                        LoginPage(username=username_data)
                                    }
                                }
                                ApplicationRoute::Signup => {
                                    view! {
                                        cx,
                                        SignupPage()
                                    }
                                }
                                a => {
                                    eprintln!("{a:?}");
                                    view! {
                                        cx,
                                        "NOT FOUND"
                                    }
                                }
                            }
                        )
                    }
                }
            }
        )
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    sycamore::render(|cx| start_application(cx))
}
