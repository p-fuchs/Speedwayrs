use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::Indexed;
use sycamore::reactive::{Signal, create_signal, create_selector};
use sycamore::{view, web::Html, reactive::Scope, view::View};
use sycamore::Prop;
use serde::{Serialize, Deserialize};

use crate::{fetch_get, fetch_json_data};

#[derive(Prop)]
pub struct GameInfoProp<'a> {
    username: &'a Signal<Option<String>>
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
struct MatchInfo {
    team1: String,
    team1_id: i32,
    team2: String,
    team2_id: i32,
    date: time::OffsetDateTime,
    score: String
}

const LAST_GAMES_ENDPOINT: &str = const_format::formatcp!("{}/data/last_games", crate::SERVER_ADDRESS);
const LIKED_TEAMS_ENDPOINT: &str = const_format::formatcp!("{}/data/liked_teams", crate::SERVER_ADDRESS);
const LIKED_PLAYERS_ENDPOINT: &str = const_format::formatcp!("{}/data/liked_players", crate::SERVER_ADDRESS);

pub fn GamesPage<'a, G: Html>(cx: Scope<'a>, info: GameInfoProp<'a>) -> View<G> {
    let page: &Signal<i32> = create_signal(cx, 1);
    let last_games: &Signal<Option<Vec<MatchInfo>>> = create_signal(cx, None);
    let liked_teams: &Signal<Option<Vec<MatchInfo>>> = create_signal(cx, None);
    let liked_players: &Signal<Option<Vec<MatchInfo>>> = create_signal(cx, None);

    let is_logged = create_selector(cx, || {
        info.username.get().as_ref().is_some()
    });

    let fetch_last_games = move || async {
        let body = serde_json::json!({
            "page": *page.get()
        });

        last_games.set(fetch_json_data(LAST_GAMES_ENDPOINT, &body).await);
    };

    let fetch_favourites = move || async {
        liked_teams.set(fetch_get(LIKED_TEAMS_ENDPOINT).await);
        liked_players.set(fetch_get(LIKED_PLAYERS_ENDPOINT).await);
    };

    let next_page = move |_| {
        page.set(*page.get() + 1);

        spawn_local_scoped(cx, async move { fetch_last_games().await });
    };

    let previus_page = move |_| {
        let prev = *page.get();

        if prev != 1 {
            page.set(prev - 1);
        }

        spawn_local_scoped(cx, async move { fetch_last_games().await });
    };

    if *is_logged.get() {
        spawn_local_scoped(cx, async move { fetch_favourites().await });
    }

    let last_games_iterable = create_selector(cx, || last_games.get().as_ref().clone().unwrap_or_else(|| Vec::new()));
    let liked_teams_iterable = create_selector(cx, || liked_teams.get().as_ref().clone().unwrap_or_else(|| Vec::new()));
    let liked_players_iterable = create_selector(cx, || liked_players.get().as_ref().clone().unwrap_or_else(|| Vec::new()));

    spawn_local_scoped(cx, async move {
        fetch_last_games().await;

        if info.username.get().is_some() {
            fetch_favourites().await
        }
    });

    view! {
        cx,
        div(class="h-full w-full grid grid-rows-2 grid-cols-2 bg-indigo-200 justify-center") {
            div(class="flex col-span-2 row-span-1 p-3 justify-center") {
                table(class="relative border-separate border-spacing-2 border border-2 border-double border-indigo-900 text-center") {
                    thead() {
                        tr() {
                            th() {
                                "Drużyna 1"
                            }
                            th() {
                                "Wynik"
                            }
                            th() {
                                "Drużyna 2"
                            }
                            th() {
                                "Data"
                            }
                        }
                    }
                    tbody() {
                        Indexed(
                            iterable = last_games_iterable,
                            view = |cx, game| view! {cx,
                                tr() {
                                    td(class="border border-indigo-700 hover:text-green-650 p-3") {
                                        a(class="hover:text-green-600", href=format!("/team/{}", game.team1_id)) {
                                            (game.team1)
                                        }
                                    }
                                    td(class="border border-indigo-700 p-3") {
                                        (game.score)
                                    }
                                    td(class="border border-indigo-700 p-3") {
                                        a(class="hover:text-green-600", href=format!("/team/{}", game.team2_id)) {
                                            (game.team2)
                                        }
                                    }
                                    td(class="border border-indigo-700 p-3") {
                                        (game.date)
                                    }
                                }
                            }
                        )
                    }
                }
                div(class="flex flex-row absolute bordered top-auto right-10") {
                    img(src="https://www.svgrepo.com/show/488500/arrow-left.svg", heigh=50, width=50, on:click=previus_page) {

                    }
                    a(class="bordered") {
                        (*page.get())
                    }
                    img(src="https://www.svgrepo.com/show/488501/arrow-right.svg", heigh=50, width=50, on:click=next_page) {

                    }
                }
            }
            
        (
            if *is_logged.get() {
                view! {
                    cx,
                    div(class="col-span-1 row-span-1 p-4 text-center") {
                        a(class="text-bold") {
                            "Mecze Twoich ulubionych drużyn"
                        }
                        table(class="relative border-separate border-spacing-2 border border-2 border-double border-indigo-900 text-center") {
                            thead() {
                                tr() {
                                    th() {
                                        "Drużyna 1"
                                    }
                                    th() {
                                        "Wynik"
                                    }
                                    th() {
                                        "Drużyna 2"
                                    }
                                    th() {
                                        "Data"
                                    }
                                }
                            }
                            tbody() {
                                Indexed(
                                    iterable = liked_teams_iterable,
                                    view = |cx, game| view! {cx,
                                        tr() {
                                            td(class="border border-indigo-700 hover:text-green-650 p-3") {
                                                a(class="hover:text-green-600", href=format!("/team/{}", game.team1_id)) {
                                                    (game.team1)
                                                }
                                            }
                                            td(class="border border-indigo-700 p-3") {
                                                (game.score)
                                            }
                                            td(class="border border-indigo-700 p-3") {
                                                a(class="hover:text-green-600", href=format!("/team/{}", game.team2_id)) {
                                                    (game.team2)
                                                }
                                            }
                                            td(class="border border-indigo-700 p-3") {
                                                (game.date)
                                            }
                                        }
                                    }
                                )
                            }
                        }
                    }
                    div(class="col-span-1 row-span-1 p-4 text-center") {
                        a(class="text-bold") {
                            "Mecze Twoich ulubionych graczy"
                        }
                        table(class="relative border-separate border-spacing-2 border border-2 border-double border-indigo-900 text-center") {
                            thead() {
                                tr() {
                                    th() {
                                        "Drużyna 1"
                                    }
                                    th() {
                                        "Wynik"
                                    }
                                    th() {
                                        "Drużyna 2"
                                    }
                                    th() {
                                        "Data"
                                    }
                                }
                            }
                            tbody() {
                                Indexed(
                                    iterable = liked_players_iterable,
                                    view = |cx, game| view! {cx,
                                        tr() {
                                            td(class="border border-indigo-700 hover:text-green-650 p-3") {
                                                a(class="hover:text-green-600", href=format!("/team/{}", game.team1_id)) {
                                                    (game.team1)
                                                }
                                            }
                                            td(class="border border-indigo-700 p-3") {
                                                (game.score)
                                            }
                                            td(class="border border-indigo-700 p-3") {
                                                a(class="hover:text-green-600", href=format!("/team/{}", game.team2_id)) {
                                                    (game.team2)
                                                }
                                            }
                                            td(class="border border-indigo-700 p-3") {
                                                (game.date)
                                            }
                                        }
                                    }
                                )
                            }
                        }

                    }

                    
                }
            } else {
                view! {cx, }
            }
        )

        }
    }
}
