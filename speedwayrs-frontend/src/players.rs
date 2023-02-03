use std::rc::Rc;

use log::info;
use serde::Deserialize;
use sycamore::{
    futures::spawn_local_scoped,
    prelude::Indexed,
    reactive::{create_signal, Scope, Signal, create_selector},
    view,
    view::View,
    web::Html, Prop,
};

use crate::{ApplicationData, fetch_json_data};

#[derive(Deserialize, Debug, Clone)]
struct Player {
    name: String,
    sname: String,
    id: i32,
}

const PLAYER_SEARCH: &'static str =
    const_format::formatcp!("{}/data/players", crate::SERVER_ADDRESS);

async fn search_request(player: String) -> Result<Vec<Player>, ()> {
    let request = gloo_net::http::Request::post(PLAYER_SEARCH)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&serde_json::json!({ "player_name": player })).unwrap());

    match crate::client::execute(request).await {
        Err(e) => {
            log::error!("Post search team request failed: {:?}", e);

            Err(())
        }
        Ok(response) => {
            if response.status() == 500 {
                Err(())
            } else {
                match response.text().await {
                    Err(e) => {
                        log::error!("Search team response failed: {:?}", e);

                        Err(())
                    }
                    Ok(text) => {
                        log::trace!("Completed search_request().");
                        Ok(serde_json::from_str(&text).unwrap())
                    }
                }
            }
        }
    }
}

pub fn PlayersPage<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let player_name: &Signal<String> = create_signal(cx, String::new());
    let search_result: &Signal<Option<Vec<Player>>> = create_signal(cx, None);
    let error_occurred: &Signal<bool> = create_signal(cx, false);

    let serach_button = move |_| {
        error_occurred.set(false);
        search_result.set(None);

        spawn_local_scoped(cx, async move {
            match search_request(player_name.get_untracked().as_ref().into()).await {
                Ok(vec) => {
                    search_result.set(Some(vec));
                }
                Err(()) => {
                    error_occurred.set(true);
                }
            }
        })
    };

    let render_table = move |players: Rc<Option<Vec<Player>>>| {
        let players_ref = players.as_ref();

        match players_ref.as_ref() {
            None => {
                view! {
                    cx,
                }
            }
            Some(vec) => {
                let table_fragment = View::new_fragment(vec.iter().map(|x| {
                    let name = x.name.to_string();
                    let surname = x.sname.to_string();
                    let id = x.id;

                    view! {cx, 
                        tr() {
                            td(class="border-separate border border-slate-400 w-80 shadow-sm bg-indigo-100 text-center") {
                                a(class="hover:text-sky-700", href=format!("/player/{}", id)) {
                                    (format!("{name} {surname}"))
                                }
                            }
                        }
                    }
                }).collect());

                view! {
                    cx,
                    table(class="border-separate border border-slate-700 w-80 shadow-sm bg-indigo-400 text-center") {
                        thead() {
                            tr() {
                                th(class="border-separate border border-slate-600 text-center") {
                                    "Player"
                                }
                            }
                        }
                    }
                    tbody() {
                        (table_fragment)
                    }
                }
            }
        }
    };

    view! {
        cx,
        div(class="h-screen w-screen bg-indigo-200 items-center justify-center") {
            div(class="columns-1 flex-col flex flex-wrap items-center justify-center") {
                form(class="items-center flex-col justify-center mt-5") {
                    label(class="w-full mt-5", for="team") {
                        "Player name"
                    }

                    br() {}

                    input(
                        class="placeholder:italic rounded-md shadow-inner p-3 mt-2 mb-4",
                        type="text",
                        size="30",
                        name="player",
                        placeholder="Player name",
                        id="player",
                        bind:value=player_name) {

                    }
                }

                button(class="relative bg-indigo-300 mt-0 w-40 h-10 hover:bg-indigo-500", on:click=serach_button) {
                    "Search"
                }

                (
                    if *error_occurred.get() {
                        view! {
                            cx,
                            div() {
                                text() {
                                    "Error occured while contacting server. Please try again later."
                                }
                            }
                        }
                    } else {
                        view! {cx, }
                    }
                )
            }

            br() {}

            div(class="columns-1 flex-col flex flex-wrap items-center justify-center") {
                (
                    render_table(search_result.get())
                )
            }
        }
    }
}

const PLAYER_INFO_ENDPOINT: &str = const_format::formatcp!("{}/data/player_info", crate::SERVER_ADDRESS);
const PLAYER_LIKE_ENDPOINT: &str = const_format::formatcp!("{}/utils/like", crate::SERVER_ADDRESS);

#[derive(Prop)]
pub struct PlayerPageProps<'a> {
    player_id: i32,
    username: &'a Signal<Option<String>>
}

#[derive(Deserialize, Clone)]
struct PlayerInfo {
    three_points: u32,
    two_points: u32,
    one_points: u32,
    zero_points: u32,
    stars: u32,
    accidents: u32,
    former_teams: Vec<(i32, String, u32)>,
    name: String,
    user_like: Option<bool>
}

async fn get_player_info(player_id: i32, info: &Signal<Option<PlayerInfo>>) {
    let body = serde_json::json!({
        "player": player_id
    });

    info.set(fetch_json_data(PLAYER_INFO_ENDPOINT, &body).await);
}

#[derive(Deserialize)]
struct LikeResponse {
    team_like: Option<bool>,
    player_like: Option<bool>
}

async fn post_like(player_id: i32, player_info: &Signal<Option<PlayerInfo>>) {
    let body = serde_json::json!({
        "player_id": player_id
    });

    let response: Option<LikeResponse> = fetch_json_data(PLAYER_LIKE_ENDPOINT, &body).await;

    if let Some(info) = response {
        let player_new_info = player_info.get().as_ref().clone();

        if let Some(mut player_new_info) = player_new_info {
            player_new_info.user_like = info.player_like;

            player_info.set(Some(player_new_info));
        }
    }
}

const DESC_CSS: &str = "border border-indigo-800 p-3 bg-indigo-700/50";
const VAL_CSS: &str = "border border-indigo-600 p-3";

pub fn PlayerPage<'a, G: Html>(cx: Scope<'a>, props: PlayerPageProps<'a>) -> View<G> {
    let player_info = create_signal(cx, None);

    spawn_local_scoped(cx, async move {
        get_player_info(props.player_id, player_info).await;
    });

    let points_mean = create_selector(cx, || {
        let mean = match player_info.get().as_ref() {
            None => 0.0,
            Some(info) => {
                let counter = 3*info.three_points + 2*info.two_points + info.one_points;
                let denominator = info.three_points + info.two_points + info.one_points + info.zero_points;

                if denominator == 0 {
                    0.0
                } else {
                    (counter as f64) / (denominator as f64)
                }
            }
        };

        format!("{:.3}", mean)
    });

    let iterable_history = create_selector(cx, || {
        match player_info.get().as_ref() {
            None => Vec::new(),
            Some(info) => {
                info.former_teams.clone()
            }
        }
    });

    let like_selector = create_selector(cx, || {
        match player_info.get().as_ref() {
            None => None,
            Some(info) => {
                info.user_like.clone()
            }
        }
    });

    let update_like = move |_| {
        spawn_local_scoped(cx, async move {
            post_like(props.player_id, player_info).await;
        })
    };

    view! {
        cx,
        div(class="flex flex-col bg-indigo-200 h-screen w-screen static") {
            (
                if let Some(info) = player_info.get().as_ref().clone() {
                    view! {
                        cx,
                        div(class="grid grid-rows-auto grid-cols-2 h-3/4 w-full") {
                            div(class="row-span-1 col-span-2 p-5 text-center") {
                                a(class="text-6xl underline font-black align-left") {
                                    (info.name)
                                }
                            }
                            div(class="flex row-span-auto cols-span-1 p-3 justify-center") {
                                table(class="border border-separate border-spacing-2 border border-2 border-double border-indigo-900 p-5 text-center") {
                                    tr() {
                                        td(class=DESC_CSS) {
                                            "Zdobycie trzech punktów"
                                        }
                                        td(class=VAL_CSS) {
                                            (info.three_points)
                                        }
                                    }
                                    tr() {
                                        td(class=DESC_CSS) {
                                            "Zdobycie dwóch punktów"
                                        }
                                        td(class=VAL_CSS) {
                                            (info.two_points)
                                        }
                                    }
                                    tr() {
                                        td(class=DESC_CSS) {
                                            "Zdobycie jednego punktu"
                                        }
                                        td(class=VAL_CSS) {
                                            (info.one_points)
                                        }
                                    }
                                    tr() {
                                        td(class=DESC_CSS) {
                                            "Zdobycie zera punktów"
                                        }
                                        td(class=VAL_CSS) {
                                            (info.zero_points)
                                        }
                                    }
                                    tr() {
                                        td(class=DESC_CSS) {
                                            "Zdobycie gwiazd"
                                        }
                                        td(class=VAL_CSS) {
                                            (info.stars)
                                        }
                                    }
                                    tr() {
                                        td(class=DESC_CSS) {
                                            "Wypadki"
                                        }
                                        td(class=VAL_CSS) {
                                            (info.accidents)
                                        }
                                    }
                                    tr() {
                                        td(class=DESC_CSS) {
                                            "Średnia liczba zdobytych punktów"
                                        }
                                        td(class=VAL_CSS) {
                                            (*points_mean.get())
                                        }
                                    }
                                }     
                            }
                            div(class="flex justify-center row-span-1 p-3 cols-span-1") {
                                table(class="grow-0 border border-separate border-spacing-2 border border-2 border-double border-indigo-900 p-5 text-center") {
                                    thead() {
                                        tr() {
                                            th(class=DESC_CSS) {
                                                "Nazwa drużyny"
                                            }
                                            th(class=DESC_CSS) {
                                                "Ilość zagranych meczy"
                                            }
                                        }
                                    }
                                    tbody() {
                                        Indexed(
                                            iterable=iterable_history,
                                            view = |cx, data| view! {
                                                cx,
                                                tr() {
                                                    td(class="border border-indigo-600 p-3") {
                                                        a(class="hover:text-green-600", href=format!("/team/{}", data.0)) {
                                                            (data.1)
                                                        }
                                                    }
                                                    td(class=VAL_CSS) {
                                                        (data.2)
                                                    }
                                                }
                                            }
                                        )
                                    }
                                }
                            }
                        }
                    }
                } else {
                    view! {
                        cx,
                    
                    }
                }
            )
            div(class="h-auto w-auto") {}
            (
                {
                    match like_selector.get().as_ref() {
                        None => {
                            log::debug!("LIKED NONE");
                            view! {cx , }
                        },
                        Some(liked) => {
                            log::debug!("LIKED {liked}");
                            if !liked {
                                view! {
                                    cx,
                                    div(class="absolute right-10") {
                                        img(class="cursor-pointer", src="https://i.imgur.com/8XUePqB.png", width=50, heigh=50, on:click=update_like) {}
                                    }
                                }
                            } else {
                                view! {
                                    cx,
                                    div(class="absolute right-10") {
                                        img(class="cursor-pointer", src="https://i.imgur.com/QhLKbOT.png", width=50, heigh=50, on:click=update_like) {}
                                    }
                                }
                            }
                        }
                    }
                }
            )
        }
    }
}
