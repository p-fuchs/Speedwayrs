mod client;
mod login;
mod match_info;
mod navbar;
mod players;
mod signup;
mod teams;
mod utils;

use login::LoginPage;
use match_info::MatchInfo;
use navbar::Navbar;
use players::PlayersPage;
use signup::SignupPage;
use sycamore::{
    futures::spawn_local_scoped,
    reactive::{create_signal, ReadSignal, Scope, Signal},
    view,
    view::View,
    web::Html,
    Prop,
};
use sycamore_router::{HistoryIntegration, Route, Router};
use teams::{TeamInfoPage, TeamsPage};
pub use utils::fetch_json_data;

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
    Team { team_id: i32 },
    #[to("/players")]
    Players,
    #[to("/match/<match_id>")]
    Match { match_id: i32 },
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
                                ApplicationRoute::Players => {
                                    view! {
                                        cx,
                                        PlayersPage()
                                    }
                                }
                                ApplicationRoute::Team {team_id} => {
                                    view! {
                                        cx,
                                        TeamInfoPage(username=username_data, team_id=*team_id)
                                    }
                                }
                                ApplicationRoute::Match {match_id} => {
                                        view! {
                                            cx,
                                            MatchInfo(match_id=*match_id)
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
