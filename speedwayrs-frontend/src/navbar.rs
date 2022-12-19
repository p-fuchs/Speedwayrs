use crate::ApplicationData;
use gloo_net::http::Request;
use sycamore::{
    component,
    reactive::{create_selector, Scope},
    view,
    view::View,
    web::Html, futures::spawn_local_scoped,
};
use sycamore_router::navigate;

const LOGOUT_ADDRESS: &'static str = const_format::formatcp!("{}/users/logout", crate::SERVER_ADDRESS);

async fn logout() {
    let request = Request::post(LOGOUT_ADDRESS);

    let _ = crate::client::execute(request).await;
}

#[component]
pub fn Navbar<'a, G: Html>(cx: Scope<'a>, data: ApplicationData<'a>) -> View<G> {
    let username_present = create_selector(cx, move || data.get_username().get().is_some());

    let logout_button = move |_| {
        spawn_local_scoped(cx, async move {
            logout().await;
            crate::client::update_session_info(cx, data.username).await;
            navigate("/home");
        });

        
    };

    view! {
        cx,
        section(class="bg-indigo-900 w-full text-white flex flex-row text-xl font-sans font-semibold") {
            div(class="p-3 pr-6") {
                img(width="80", height="80", src="https://i.imgur.com/hrbJ4I3.png")
            }

            nav(class="flex flex-auto space-x-10 items-center") {
                a(class="hover:text-red-700", href="/") {
                    "Home"
                }
                a(class="hover:text-red-700", href="/games") {
                    "Games"
                }
                a(class="hover:text-red-700", href="/chat") {
                    "Chat"
                }
            }

            div(class="flex pr-6 space-x-10 items-center") {
                (
                    if *username_present.get() {
                        let data_username = data.username.get();
                        let username_ref = data_username.as_ref().as_ref().unwrap().to_owned();

                        view! {cx,
                            a(class="hover:text-green-700", href="/dashboard") {
                                "Dashboard"
                            }
                            button(class="hover:text-rose-700", on:click=logout_button) {
                                "Logout"
                            }
                            a(class="text-yellow-500 italic") {
                                (username_ref)
                            }
                        }
                    } else {
                        view! {cx,
                            a(class="hover:text-green-700", href="/login") {
                                "Login"
                            }
                            a(class="hover:text-green-700", href="/signup") {
                                "Signup"
                            }
                        }
                    }
                )
            }
        }
    }
    /*cx.render(rsx! (
        section {
            class: "bg-indigo-900 w-full text-white flex flex-row text-xl font-sans font-semibold",

            div {
                class: "p-3 pr-6",
                img {
                    width: "80",
                    height: "80",
                    src: "https://i.imgur.com/hrbJ4I3.png"
                }
            }

            nav {
                class: "flex flex-auto space-x-10 items-center",
                Link {
                    to: "/",
                    class: "hover:text-red-700",
                    "Home"
                }
                Link {
                    to: "/games",
                    class: "hover:text-red-700",
                    "Games"
                }
                Link {
                    to: "/chat",
                    class: "hover:text-red-700",
                    "Chat"
                }
            }

            div {
                class: "flex pr-6 space-x-10 items-center",
                Link {
                    to: "/login",
                    class: "hover:text-lime-700",
                    "Login"
                }
                Link {
                    to: "/signup",
                    class: "hover:text-lime-700",
                    "Signup"
                }
            }
        }
    ))*/
}
