use std::rc::Rc;

use log::info;
use serde::Deserialize;
use sycamore::{web::Html, view::View, view, reactive::{Scope, Signal, create_signal}, futures::spawn_local_scoped, prelude::Indexed};

use crate::ApplicationData;

#[derive(Deserialize, Debug, Clone)]
struct Team {
    name: String,
    id: i32
}

const TEAM_SEARCH: &'static str = const_format::formatcp!("{}/data/teams", crate::SERVER_ADDRESS);

async fn search_request(team: String) -> Result<Vec<Team>, ()> {
    let request = gloo_net::http::Request::post(TEAM_SEARCH)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&serde_json::json!({
            "team_name": team
        })).unwrap());

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
                        Ok(
                            serde_json::from_str(&text).unwrap()
                        )
                    }
                }
            }
        }
    }
}

pub fn TeamsPage<'a, G:Html>(cx: Scope<'a>) -> View<G> {
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