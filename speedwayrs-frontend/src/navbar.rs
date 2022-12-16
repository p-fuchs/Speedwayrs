use crate::Account;
use dioxus::prelude::*;
use dioxus::router::Link;

pub fn render_navbar(cx: Scope) -> Element {
    cx.render(rsx! (
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
    ))
}