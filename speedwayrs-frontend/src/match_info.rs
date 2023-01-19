use sycamore::{Prop, reactive::{Scope, Signal, create_signal}, web::Html, view::View, view, futures::spawn_local_scoped};
use speedwayrs_types::MatchResult;

use crate::fetch_json_data;

#[derive(Prop)]
pub struct MatchInfoParams {
    match_id: i32 
}

const MATCH_INFO_ENDPOINT: &str = const_format::formatcp!("{}/data/match_info", crate::SERVER_ADDRESS);

async fn retrieve_match_info(match_id: i32, place: &Signal<Option<MatchResult>>) {
    let body = serde_json::json!({
        "match_id": match_id
    });

    place.set(fetch_json_data(MATCH_INFO_ENDPOINT, &body).await);
}

fn show_match_info<'a, G:Html>(cx: Scope<'a>, match_info: &'a MatchResult) -> View<G> {
    view! {
        cx,
        div(class="flex flex-col") {
            div(class="flex flex-row") {
                a() {
                    (match_info.first_team_name())
                }
                a() {
                    (
                        format!("{}:{}", match_info.first_team_score(), match_info.second_team_score())  
                    )
                }
                a() {
                    (match_info.second_team_name())
                }
            }
        }
    } 
}

pub fn MatchInfo<'a, G: Html>(cx: Scope<'a>, match_info: MatchInfoParams) -> View<G> {
    let info_signal = create_signal(cx, None);

    spawn_local_scoped(cx, async move {
        retrieve_match_info(match_info.match_id, info_signal).await;
    });

    match info_signal.get().as_ref() {
        None => {
            view! {cx, }
        }
        Some(reference) => {
            show_match_info(cx, reference)
        }
    } 
}
