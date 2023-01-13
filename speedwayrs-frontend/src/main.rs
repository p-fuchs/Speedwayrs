mod login;
mod players;
mod navbar;
mod signup;
mod client;
mod teams;

use login::LoginPage;
use navbar::Navbar;
use signup::SignupPage;
use sycamore::{
    reactive::{create_signal, ReadSignal, Scope, Signal},
    view,
    view::View,
    web::Html,
    Prop, futures::spawn_local_scoped,
};
use sycamore_router::{HistoryIntegration, Route, Router};
use teams::TeamsPage;

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
    #[to("/teams")]
    Teams,
    #[to("/team/<team_id>")]
    Team {
        team_id: u16
    },
    #[to("/players")]
    Players,
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
                                ApplicationRoute::Teams => {
                                    view! {
                                        cx,
                                        TeamsPage()
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
