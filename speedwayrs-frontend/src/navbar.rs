use crate::ApplicationData;
use sycamore::{
    component,
    reactive::{create_selector, Scope},
    view,
    view::View,
    web::Html,
};

#[component]
pub fn Navbar<'a, G: Html>(cx: Scope<'a>, data: ApplicationData<'a>) -> View<G> {
    let username_present = create_selector(cx, move || data.get_username().get().is_some());

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
                        view! {cx,
                            a(class="hover:text-red-700", href="/dashboard") {
                                "Dashboard"
                            }
                        }
                    } else {
                        view! {cx,
                            a(class="hover:text-red-700", href="/login") {
                                "Login"
                            }
                            a(class="hover:text-red-700", href="/signup") {
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
