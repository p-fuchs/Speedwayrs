use speedwayrs_types::{MatchResult, PlayerResult, RunInfo};
use sycamore::{
    futures::spawn_local_scoped,
    prelude::{Children, Indexed},
    reactive::{create_selector, create_signal, Scope, Signal},
    view,
    view::View,
    web::Html,
    Prop,
};

use crate::fetch_json_data;

#[derive(Prop)]
pub struct MatchInfoParams {
    match_id: i32,
}

const MATCH_INFO_ENDPOINT: &str =
    const_format::formatcp!("{}/data/match_info", crate::SERVER_ADDRESS);

async fn retrieve_match_info(match_id: i32, place: &Signal<Option<MatchResult>>) {
    let body = serde_json::json!({ "match_id": match_id });

    let match_info =
        fetch_json_data(MATCH_INFO_ENDPOINT, &body)
            .await
            .map(|mut info: MatchResult| {
                info.sort_runs();

                info
            });

    place.set(match_info);
}

fn generate_score(info: &Signal<Option<MatchResult>>) -> String {
    match info.get().as_ref() {
        None => "".into(),
        Some(info) => {
            format!("{} : {}", info.first_team_score(), info.second_team_score())
        }
    }
}

#[derive(Prop)]
struct RunResultInfo<'a, G: Html> {
    info: RunInfo,
    children: Children<'a, G>,
}

fn RunResult<'a, G: Html>(cx: Scope<'a>, run_info: RunResultInfo<'a, G>) -> View<G> {
    let info_clone = run_info.info.clone();

    let run_score_selector = create_selector(cx, move || {
        info_clone
            .scores()
            .iter()
            .map(|(player_id, name, score)| {
                let id = *player_id;
                let name = name.clone();
                let parsed_score = match PlayerResult::from_str(score) {
                    Some(score) => score.to_pretty(),
                    None => "Error.".into(),
                };

                (id, name, parsed_score)
            })
            .collect()
    });

    let round_number = run_info.info.number();

    view! {
        cx,
        div(class="outline outline-2 outline-offset-3 p-2 m-6 rounded-md justify-items-center") {
            div(class="grid justify-items-center items-center") {
                div(class="w-1/2 grid grid-cols-2 border border-2 border-double border-indigo-900/60 bg-indigo-500/40") {
                    div(class="pl-5 col-span-1 text-left") {
                        (format!("Runda {}:", round_number))
                    }
                    div(class="col-span-1 pr-5 text-right") {
                        (run_info.info.time_string())
                    }
                }
            }
            table(class="w-full h-full border-separate border-spacing-2 text-center") {
                thead() {
                    tr() {
                        th(class="border border-2 rounded-md bg-indigo-600/40 border-indigo-900/50") {
                            "Zawodnik"
                        }
                        th(class="border border-2 rounded-md bg-indigo-600/40 border-indigo-900/50") {
                            "Wynik"
                        }
                    }
                }
                tbody() {
                    Indexed(
                        iterable=run_score_selector,
                        view = |cx, to_view| view! {
                            cx,
                            tr() {
                                td(class="border border-2 rounded-md border-indigo-300/50") {
                                    a(class="hover:text-green-700", href=format!("/player/{}", to_view.0)) {
                                        (to_view.1)
                                    }
                                }
                                td(class="border border-2 rounded-md border-indigo-300/50") {
                                    (to_view.2)
                                }
                            }
                        }
                    )
                }
            }
        }
    }
}

pub fn MatchInfo<'a, G: Html>(cx: Scope<'a>, match_info: MatchInfoParams) -> View<G> {
    let info_signal = create_signal::<Option<MatchResult>>(cx, None);

    let first_team_name = create_selector(cx, move || -> String {
        match info_signal.get().as_ref() {
            None => "".into(),
            Some(info) => info.first_team_name().into(),
        }
    });

    let second_team_name = create_selector(cx, move || -> String {
        match info_signal.get().as_ref() {
            None => "".into(),
            Some(info) => info.second_team_name().into(),
        }
    });

    let run_infos = create_selector(cx, move || match info_signal.get().as_ref() {
        None => Vec::new(),
        Some(info) => info.runs().to_vec(),
    });

    spawn_local_scoped(cx, async move {
        retrieve_match_info(match_info.match_id, info_signal).await;
    });

    view! {
        cx,
        div(class="grid grid-cols-2 auto-rows-max w-full h-full bg-indigo-200") {
            div(class="col-span-2 row-span-1 flex flex-col w-full h-full pt-7") {
                div(class="grid grid-cols-5 w-full") {
                    a(class="col-span-2 text-3xl font-semibold text-center") {
                        span() {
                            (first_team_name.get())
                        }
                    }
                    a(class="col-span-1 font-serif text-7xl font-semibold") {
                        p(class="text-center") {
                            (
                                generate_score(info_signal)
                            )
                        }
                    }
                    a(class="col-span-2 text-3xl font-semibold text-center") {
                        (second_team_name.get())
                    }
                }
            }
            Indexed(
                iterable=run_infos,
                view = |cx, run| view! {
                    cx,
                    div(class="w-full") {
                        RunResult(info=run) {}
                    }
                }
            )
        }
    }
}
