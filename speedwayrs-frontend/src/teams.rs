use std::rc::Rc;

use log::info;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sycamore::{
    futures::spawn_local_scoped,
    prelude::Indexed,
    reactive::{create_selector, create_signal, Scope, Signal},
    view,
    view::View,
    web::Html,
    Prop,
};

use crate::utils::fetch_json_data;
use crate::ApplicationData;

#[derive(Deserialize, Debug, Clone)]
struct Team {
    name: String,
    id: i32,
}

const TEAM_SEARCH: &'static str = const_format::formatcp!("{}/data/teams", crate::SERVER_ADDRESS);
const TEAM_INFO: &'static str = const_format::formatcp!("{}/data/team_info", crate::SERVER_ADDRESS);
const TEAM_LIKE: &'static str = const_format::formatcp!("{}/utils/like", crate::SERVER_ADDRESS);
const TEAM_STATS: &'static str =
    const_format::formatcp!("{}/data/team_stats", crate::SERVER_ADDRESS);

async fn search_request(team: String) -> Result<Vec<Team>, ()> {
    let request = gloo_net::http::Request::post(TEAM_SEARCH)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&serde_json::json!({ "team_name": team })).unwrap());

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
                    Ok(text) => Ok(serde_json::from_str(&text).unwrap()),
                }
            }
        }
    }
}

pub fn TeamsPage<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let team_name: &Signal<String> = create_signal(cx, String::new());
    let search_result: &Signal<Option<Vec<Team>>> = create_signal(cx, None);
    let error_occurred: &Signal<bool> = create_signal(cx, false);

    let serach_button = move |_| {
        error_occurred.set(false);
        search_result.set(None);

        spawn_local_scoped(cx, async move {
            match search_request(team_name.get_untracked().as_ref().into()).await {
                Ok(vec) => {
                    info!("GOT {vec:?}");
                    search_result.set(Some(vec));
                }
                Err(()) => {
                    error_occurred.set(true);
                }
            }
        })
    };

    let render_table = move |teams: Rc<Option<Vec<Team>>>| {
        let teams_ref = teams.as_ref();

        match teams_ref.as_ref() {
            None => {
                view! {
                    cx,
                }
            }
            Some(vec) => {
                let table_fragment = View::new_fragment(vec.iter().map(|x| {
                    let name = x.name.to_string();
                    let id = x.id;

                    view! {cx, 
                        tr() {
                            td(class="border-separate border border-slate-400 w-80 shadow-sm bg-indigo-100 text-center") {
                                a(class="hover:text-sky-700", href=format!("/team/{}", id)) {
                                    (name)
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
                                    "Team"
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
                        "Team name"
                    }

                    br() {}

                    input(
                        class="placeholder:italic rounded-md shadow-inner p-3 mt-2 mb-4",
                        type="text",
                        size="30",
                        name="team",
                        placeholder="Team name",
                        id="team",
                        bind:value=team_name) {

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
                                    "a"
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

#[derive(Prop, Copy, Clone)]
pub struct TeamInfo<'a> {
    username: &'a Signal<Option<String>>,
    team_id: i32,
}

#[derive(Deserialize, PartialEq, Clone)]
struct MatchInfo {
    match_id: i32,
    opponent_name: String,
    opponent_id: i32,
    date: String,
}

#[derive(Deserialize, Clone)]
struct ResponseMatchInfo {
    team_name: String,
    last_matches: Vec<MatchInfo>,
    user_like: Option<bool>,
}

impl ResponseMatchInfo {
    pub fn team_name(&self) -> &str {
        &self.team_name
    }
}

const PAGE_SIZE: u32 = 10;

async fn request_info(
    team_id: i32,
    response: &Signal<Option<ResponseMatchInfo>>,
    connection_error: &Signal<bool>,
    page: u32,
) {
    let request = gloo_net::http::Request::post(TEAM_INFO)
        .header("Content-Type", "application/json")
        .body(
            &serde_json::json!({
                "team_id": team_id,
                "skip_first": PAGE_SIZE * (page - 1),
                "step": PAGE_SIZE
            })
            .to_string(),
        );

    let query_result = crate::client::execute(request).await;

    match query_result {
        Ok(query_response) => match query_response.text().await {
            Ok(body) => match serde_json::from_str::<ResponseMatchInfo>(&body) {
                Ok(team_info) => {
                    response.set(Some(team_info));
                }
                Err(e) => {
                    log::error!("Unable to parse server team_info response. Error = [{e:?}]");
                }
            },
            Err(e) => {
                log::error!("Unable to get text body of team_info response. Error = [{e:?}]");

                connection_error.set(true);
            }
        },
        Err(query_error) => {
            log::error!(
                "Unable to execute request of team_info function. Error = [{query_error:?}]"
            );

            connection_error.set(true)
        }
    }
}

#[derive(Deserialize)]
struct PostLikeResponse {
    team_like: Option<bool>,
}

async fn post_like(team_id: i32, team_info: &Signal<Option<ResponseMatchInfo>>) {
    let request = gloo_net::http::Request::post(TEAM_LIKE)
        .header("Content-Type", "application/json")
        .body(&serde_json::json!({ "team_id": team_id }).to_string());

    let response = crate::client::execute(request).await;

    match response {
        Err(e) => {
            log::error!("Like response returned error. Error = [{e:?}]");
        }
        Ok(response) => {
            if response.status() == http::StatusCode::OK {
                match response.text().await {
                    Ok(body) => match serde_json::from_str::<PostLikeResponse>(&body) {
                        Ok(like_response) => {
                            let info_ref = team_info.get();
                            let mut info_clone = info_ref.as_ref().clone();

                            if let Some(info) = &mut info_clone {
                                info.user_like = like_response.team_like;

                                team_info.set(info_clone);
                            }
                        }
                        Err(e) => {
                            log::error!("Unable to deserialize PostLikeResponse. Error = [{e:?}]");
                        }
                    },
                    Err(e) => {
                        log::error!("Unable to get text body of response. Error = [{e:?}]");
                    }
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
struct GameData {
    id: i32,
    opponent: String,
    games: u32,
}

#[derive(Debug, Deserialize, Clone)]
struct TeamStats {
    wins: u32,
    looses: u32,
    ties: u32,
    often_looses: Vec<GameData>,
    often_wins: Vec<GameData>,
}

async fn update_team_stats(team_id: i32, team_stats: &Signal<Option<TeamStats>>) {
    let req_body = serde_json::json!({ "team_id": team_id });

    if let Some(stats) = fetch_json_data(TEAM_STATS, &req_body).await {
        team_stats.set(stats);
    }
}

pub fn TeamInfoPage<'a, G: Html>(cx: Scope<'a>, info: TeamInfo<'a>) -> View<G> {
    let page = create_signal(cx, 1u32);
    let connection_error = create_signal(cx, false);
    let team_info = create_signal(cx, None as Option<ResponseMatchInfo>);
    let team_stats = create_signal(cx, None as Option<TeamStats>);

    let update_info = move || {
        spawn_local_scoped(cx, async move {
            request_info(
                info.team_id,
                team_info,
                connection_error,
                *page.get().as_ref(),
            )
            .await;
        })
    };

    spawn_local_scoped(cx, async move {
        update_team_stats(info.team_id, team_stats).await;
    });

    update_info();

    let plus_button = move |_| {
        *page.modify() += 1;

        update_info();
    };

    let minus_button = move |_| {
        if *page.get_untracked().as_ref() != 1 {
            *page.modify() -= 1;

            update_info();
        }
    };

    let iterable_match_info = create_selector(cx, move || match team_info.get().as_ref() {
        None => Vec::new(),
        Some(vec) => vec.last_matches.clone(),
    });

    let iterable_lost_games = create_selector(cx, move || match team_stats.get().as_ref() {
        None => Vec::new(),
        Some(vec) => vec.often_looses.clone(),
    });

    let iterable_won_games = create_selector(cx, move || match team_stats.get().as_ref() {
        None => Vec::new(),
        Some(vec) => vec.often_wins.clone(),
    });

    let like_selector = create_selector(cx, move || match team_info.get().as_ref() {
        Some(info) => info.user_like,
        None => None,
    });

    let update_like = move |_| {
        log::debug!("Updating like");
        spawn_local_scoped(cx.clone(), async move {
            post_like(info.team_id, team_info).await;
        });
    };

    view! {
        cx,
        div(class="h-screen w-screen bg-indigo-200") {
            br() {}

            div(class="flex flex-wrap flex-col items-center ml-4 mt-4 mb-15 static") {
                a(class="font-sans text-6xl ml-20 font-black underline tracking-wide text-left text-indigo-800 decoration-double") {
                    (
                        {
                            match team_info.get().as_ref() {
                                Some(info) => {
                                    info.team_name().to_string()
                                }
                                None => {
                                    "".into()
                                }
                            }
                        }
                    )
                }
                (
                    {
                        match like_selector.get().as_ref() {
                            Some(liked) => {
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
                            None => {
                                view! {
                                    cx,
                                }
                            }
                        }
                    }
                )
            }

            div(class="flex flex-row flex-nowrap w-screen h-screen") {
                div(class="flex flex-col basis-1/2 items-center text-lg") {
                    a(class="font-extrabold") {
                        "Statystyki"
                    }
                    (
                        {
                            let team_stats_struct = team_stats.get().as_ref().clone();
                            match team_stats_struct {
                                Some(stats) => {
                                    view! {
                                        cx,
                                        div(class="border-4 border-double my-15 w-8/12 border-indigo-900/30") {
                                            table(class="border-separate w-full mb-15 table-auto border-spacing-0.5") {
                                                tbody(class="font-normal") {
                                                    tr() {
                                                        th(class="rounded-none border-2 border-indigo-500/20") {
                                                            (
                                                                "Wygrane"
                                                            )
                                                        }
                                                        th(class="rounded-none border-2 border-indigo-500/20") {
                                                            (
                                                                stats.wins
                                                            )
                                                        }
                                                    }
                                                    tr() {
                                                        th(class="rounded-none border-2 border-indigo-500/20") {
                                                            (
                                                                "Przegrane"
                                                            )
                                                        }
                                                        th(class="rounded-none border-2 border-indigo-500/20") {
                                                            (
                                                                stats.looses
                                                            )
                                                        }
                                                    }
                                                    tr() {
                                                        th(class="rounded-none border-2 border-indigo-500/20") {
                                                            (
                                                                "Remisy"
                                                            )
                                                        }
                                                        th(class="rounded-none border-2 border-indigo-500/20") {
                                                            (
                                                                stats.ties
                                                            )
                                                        }
                                                    }
                                                    tr() {
                                                        th(class="rounded-none border-2 border-indigo-500/20") {
                                                            (
                                                                "Współczynnik zwycięstwa"
                                                            )
                                                        }
                                                        th(class="rounded-none border-2 border-indigo-500/20") {
                                                            (
                                                                {
                                                                    let total = stats.wins + stats.looses + stats.ties;

                                                                    format!("{:.3}", stats.wins as f64 / total as f64)
                                                                }
                                                            )
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                None => {
                                    view! {
                                        cx,
                                    }
                                }
                            }
                        }
                    )
                    (
                        {
                            let team_stats_struct = team_stats.get().as_ref().clone();
                            match team_stats_struct {
                                None => {
                                    view! {cx, }
                                }
                                Some(_) => {
                                    if iterable_lost_games.get().as_ref().is_empty() {
                                        view! {cx ,}
                                    } else {
                                        view! {cx,
                                            a(class="pt-5 font-extrabold mt-15") {
                                                "Najczęstsze przegrane"
                                            }
                                            div(class="border-4 border-double w-8/12 m-15 border-indigo-900/30") {
                                                table(class="border-separate w-full table-auto border-spacing-0.5") {
                                                    thead(class="itaic") {
                                                        tr() {
                                                            th(class="rounded-none border-2 border-indigo-500/50") {
                                                                "Przeciwnik"
                                                            }
                                                            th(class="rounded-none border-2 border-indigo-500/50") {
                                                                "Ilość przegranych"
                                                            }
                                                        }
                                                    }
                                                    tbody(class="font-normal") {
                                                        Indexed(
                                                            iterable = iterable_lost_games,
                                                            view = |cx, lost_game| view! {
                                                                cx,
                                                                tr() {
                                                                    th(class="rounded-none border-2 border-indigo-500/20") {
                                                                        a(class="hover:text-sky-400", href=format!("/team/{}", lost_game.id)) {
                                                                            (lost_game.opponent)
                                                                        }
                                                                    }
                                                                    th(class="rounded-none border-2 border-indigo-500/20") {
                                                                        (
                                                                            lost_game.games
                                                                        )
                                                                    }
                                                                }
                                                            }

                                                            )
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    )
                    (
                        {
                            let team_stats_struct = team_stats.get().as_ref().clone();
                            match team_stats_struct {
                                None => {
                                    view! {cx, }
                                }
                                Some(_) => {
                                    if iterable_won_games.get().as_ref().is_empty() {
                                        view! {cx, }
                                    } else {
                                        view! {cx,
                                            a(class="pt-5 font-extrabold mt-15") {
                                                "Najczęstsze wygrane"
                                            }
                                            div(class="border-4 border-double w-8/12 m-15 border-indigo-900/30") {
                                                table(class="border-separate w-full table-auto border-spacing-0.5") {
                                                    thead(class="itaic") {
                                                        tr() {
                                                            th(class="rounded-none border-2 border-indigo-500/50") {
                                                                "Przeciwnik"
                                                            }
                                                            th(class="rounded-none border-2 border-indigo-500/50") {
                                                                "Ilość wygranych"
                                                            }
                                                        }
                                                    }
                                                    tbody(class="font-normal") {
                                                        Indexed(
                                                            iterable = iterable_won_games,
                                                            view = |cx, won_game| view! {
                                                                cx,
                                                                tr() {
                                                                    th(class="rounded-none border-2 border-indigo-500/20") {
                                                                        a(class="hover:text-sky-400", href=format!("/team/{}", won_game.id)) {
                                                                            (won_game.opponent)
                                                                        }
                                                                    }
                                                                    th(class="rounded-none border-2 border-indigo-500/20") {
                                                                        (
                                                                            won_game.games
                                                                        )
                                                                    }
                                                                }
                                                            }

                                                            )
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    )
                }
                div(class="flex flex-col basis-1/2 items-center text-lg") {
                    a(class="font-extrabold") {
                        "Ostatnie mecze"
                    }
                    br() {}
                    div(class="border-4 border-double w-8/12 border-indigo-900/30") {
                        table(class="border-separate w-full table-auto border-spacing-0.5") {
                            thead(class="italic") {
                                tr() {
                                    th(class="rounded-none border-2 border-indigo-500/50") {
                                        "Id meczu"
                                    }
                                    th(class="rounded-none border-2 border-indigo-500/50") {
                                        "Przeciwnik"
                                    }
                                    th(class="rounded-none border-2 border-indigo-500/50") {
                                        "Data"
                                    }
                                }
                            }
                            tbody(class="font-normal") {
                                Indexed(
                                    iterable=iterable_match_info,
                                    view=|cx, match_info| view! {
                                        cx,
                                        tr() {
                                            th(class="rounded-none border-2 border-indigo-500/20") {
                                                a(class="hover:text-sky-400", href=format!("/match/{}", match_info.match_id)) {
                                                    (
                                                        match_info.match_id
                                                    )
                                                }
                                            }
                                            th(class="rounded-none border-2 border-indigo-500/20") {
                                                a(class="hover:text-sky-400", href=format!("/team/{}", match_info.opponent_id)) {
                                                    (match_info.opponent_name)
                                                }
                                            }
                                            th(class="rounded-none border-2 border-indigo-500/20") {
                                                (
                                                    match_info.date
                                                )
                                            }
                                        }
                                    }
                                )
                            }
                        }
                    }
                    div(class="items-center flex flex-row") {
                        img(class="cursor-pointer", width=60, height=60, src="https://www.svgrepo.com/show/486206/system-arrow-left-line.svg", on:click=minus_button) {}
                        div(class="border-6 border-solid bg-indigo-800/40 border border-indigo-900/30") {
                            a(class="m-3") {
                                (
                                    *page.get().as_ref()
                                )
                            }
                        }
                        img(class="cursor-pointer", width=60, height=60, src="https://www.svgrepo.com/show/486204/system-arrow-right-line.svg", on:click=plus_button) {}
                    }
                }
            }
        }
    }
}
