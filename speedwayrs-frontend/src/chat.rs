use serde::{Serialize, Deserialize};
use sycamore::{view, web::Html, reactive::{Scope, Signal, create_signal}, Prop, view::View, futures::spawn_local_scoped, motion::create_tweened_signal, prelude::Indexed};
use time::format_description::OwnedFormatItem;

use crate::{fetch_json_data, utils::fetch_response};

#[derive(Prop)]
pub struct ChatProps<'a> {
    username: &'a Signal<Option<String>>
}

#[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
struct Message {
    username: String,
    message: String,
    time: time::OffsetDateTime
}

impl Message {
    const FORMATTER: once_cell::sync::OnceCell<OwnedFormatItem> = once_cell::sync::OnceCell::new();

    pub fn username(&self) -> String {
        format!("{}:", self.username)
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn time(&self) -> String {
        let binding = Self::FORMATTER;

        let formatter = binding.get_or_init(|| {
            time::format_description::parse_owned("[day]-[month]-[year] [hour]:[minute]").unwrap()
        });

        self.time.format(formatter).unwrap()
    }
}

#[derive(Serialize)]
struct Request {
    page: u16
}

const CHAT_CONTENT_ENDPOINT: &str = const_format::formatcp!("{}/utils/chat/messages", crate::SERVER_ADDRESS);
const POST_MESSAGE_ENDPOINT: &str = const_format::formatcp!("{}/utils/chat/post_message", crate::SERVER_ADDRESS);

async fn get_messages(page: u16, messages: &Signal<Vec<Message>>) {
    let body = Request { page };

    let response: Option<Vec<Message>> = fetch_json_data(CHAT_CONTENT_ENDPOINT, &body).await;
    log::debug!("Got reposne: {response:?}");

    if let Some(mut vec) = response {
        vec.sort_by(|x,y| y.time.cmp(&x.time));

        messages.set(vec);
    }
}

async fn post_message(message: String, post_result: &Signal<Option<&'static str>>) {
    let body = serde_json::json!({
        "message": message
    });

    let response = fetch_response(POST_MESSAGE_ENDPOINT, &body).await;

    match response {
        Ok(response) => {
            if response.status() != http::StatusCode::OK {
                log::warn!("Server responed: {:?}", response.text().await);
                post_result.set(Some("Server denied request. Are you logged in?"));
            }
        }
        Err(e) => {
            log::error!("Error while posting message. Error = [{e:?}]");
            post_result.set(Some("Error while sending request to server."));
        }
    }
}

pub fn ChatPage<'a, G: Html>(cx: Scope<'a>, props: ChatProps<'a>) -> View<G> {
    let page = create_signal(cx, 1u16);
    let message = create_signal(cx, String::new());
    let messages: &Signal<Vec<Message>> = create_signal(cx, Vec::new());
    let send_result = create_signal(cx, None);

    spawn_local_scoped(cx, async move {
        get_messages(*page.get(), messages).await;
    });

    let send_message = move |_| {
        let message = message.get().as_ref().clone();

        spawn_local_scoped(cx, async move {
            post_message(message, send_result).await;
            get_messages(*page.get(), messages).await;
        });
    };

    let reload_chat = move |_| {
        spawn_local_scoped(cx, async move {
            get_messages(*page.get(), messages).await;
        });
    };

    view! {
        cx,
        div(class="flex w-full h-full bg-indigo-200 justify-center") {
            div(class="basis-1/2 grid auto-rows-auto overflow-scroll static") {
                img(class="absolute right-0 mt-2 mr-2 cursor-pointer", width=50, heigh=50,
                    src="https://www.svgrepo.com/show/382076/reload-data-infographic-update-element-graph.svg", on:click=reload_chat) {}
                Indexed(
                    iterable=messages,
                    view = |cx, msg| {
                        let username = msg.username();
                        let time = msg.time();
                        let message = msg.message().to_string();

                        view! {
                            cx,
                            div(class="grid grid-rows-2 grid-cols-6 rounded-md border-2 border-indigo-700 border-dotted mt-2") {
                                div(class="pl-2 row-span-1 col-span-4") {
                                    (username)
                                }
                                div(class="pr-2 row-span-1 col-span-2 text-right") {
                                    (time)
                                }
                                div(class="pl-5 pr-5 row-span-1 col-span-6") {
                                    (message)
                                }
                            }
                        }
                    }
                )
                div(class="grid grid-cols-6 grid-rows-3 mb-5") {
                    input(class="col-span-full row-span-2 rounded-md shadow-inner p-3 mt-2 mb-4", bind:value=message, type="text", size="30", name="message", placeholder="Type your message...") {}
                    (   
                        if props.username.get().as_ref().is_some() {
                            view! {
                                cx,
                                button(class="row-span-1 col-span-1 bg-indigo-300 hover:bg-indigo-500", on:click=send_message) {
                                    "Send message"
                                }
                            }
                        } else {
                        view! {
                            cx,
                            button(class="row-span-1 col-span-1 bg-gray-300 hover:bg-gray-500") {
                                "Send message"
                            }
                        }
                    }
                    )
                }
            }
        }
    }    
}
