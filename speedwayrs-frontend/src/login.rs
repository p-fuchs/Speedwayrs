use sycamore::{reactive::{Scope, Signal, create_signal, create_selector}, view, view::View, web::Html};

#[derive(Debug, Clone, Copy)]
enum LoginError {
    WrongCredentials,
    ServerProblem
}

impl LoginError {
    pub fn error_title(&self) -> &'static str {
        match self {
            Self::WrongCredentials => {
                "Wrong credentials."
            }
            Self::ServerProblem => {
                "Problem with server."
            }
        }
    }

    pub fn error_description(&self) -> &'static str {
        match self {
            Self::WrongCredentials => {
                "Your e-mail address or / and password is not correct."
            }
            Self::ServerProblem => {
                "This may be temporary issue with the server. Please try again later."
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn LoginPage<G: Html>(cx: Scope) -> View<G> {
    let username: &Signal<String> = create_signal(cx, String::new());
    let password: &Signal<String> = create_signal(cx, String::new());

    let visible_error: &Signal<Option<LoginError>> = create_signal(cx, None);

    let password_length = create_selector(cx, move || password.get().len());
    let username_empty = create_selector(cx, move || username.get().is_empty());

    let submit_button = move |_| {


        log::error!("{:?}", password.clone())
    };

    let has_error = create_selector(cx, move || visible_error.get().is_some());

    view! {
        cx,
        div(class="h-screen w-screen bg-indigo-200 items-center justify-center") {
            div(class="columns-1 flex-col flex flex-wrap items-center justify-center") {
                form(class="items-center flex-col justify-center mt-5") {
                    label(class="w-full mt-5", for="username") {
                        "Username"
                    }

                    br() {}

                    input(
                        class="placeholder:italic rounded-md shadow-inner p-3 mt-2 mb-4",
                        type="text",
                        size="30",
                        name="username",
                        placeholder="Username",
                        id="username",
                        bind:value=username) {
    
                    }

                    br() {}

                    label(class="w-full mt-4", for="password") {
                        "Password"
                    }

                    br() {}
                    
                    input(
                        class="placeholder:italic rounded-md shadow-inner p-3 mt-2 mb-4",
                        type="password",
                        size="30",
                        name="password",
                        placeholder="Password",
                        id="password",
                        bind:value=password) {}

                    br() {}
                }

                button(class="relative bg-indigo-300 mt-5 w-40 h-10 hover:bg-indigo-500", on:click=submit_button) {
                    "Login"
                }

                (
                    if *has_error.get() {
                        view! {
                            cx,
                            div(class="rounded-lg border-2 border-rose-500/75 mt-10 bg-rose-200") {
                                p(class="px-2 text-xl subpixel-antialiased font-extrabold text-center") {
                                    (visible_error.get().unwrap().error_title())
                                }

                                a(class="container px-2 text-m subpixel-antialiased") {
                                    (visible_error.get().unwrap().error_description())
                                }
                            }
                        }
                    } else {
                        view! {
                            cx,
                        }
                    }
                )
            }
        }
    }
}

/*
pub fn login_page(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            class: "h-screen w-screen bg-indigo-200 items-center justify-center",
            div {
                class: "columns-1 flex-col flex flex-wrap items-center justify-center",

                form {
                    class: "items-center flex-col justify-center mt-5",
                    label {
                        class: "w-full mt-5",
                        r#for: "username",
                        "Username"
                    }
                    br {}
                    input {
                        class: "placeholder:italic rounded-md shadow-inner p-3 mt-2 mb-4",
                        r#type: "text",
                        size: "30",
                        name: "username",
                        placeholder: "Username",
                        id: "username"
                    }
                    br {}
                    label {
                        class: "mt-4",
                        r#for: "email",
                        "E-mail"
                    }
                    br {}
                    input {
                        class: "placeholder:italic rounded-md shadow-inner p-3 mt-2 mb-4",
                        r#type: "text",
                        size: "30",
                        name: "email",
                        placeholder: "E-mail",
                        id: "email"
                    }
                    br {}
                    label {
                        class: "mt-4",
                        r#for: "password",
                        "Password"
                    }
                    br {}
                    input {
                        class: "placeholder:italic rounded-md shadow-inner p-3 mt-2",
                        r#type: "password",
                        size: "30",
                        name: "password",
                        placeholder: "Password",
                        id: "password"
                    }
                    br {}
                    button {
                        class: "relative bg-indigo-50 mt-5 w-30",
                        r#type: "submit",
                        "Submit form"
                    }
                }
            }
        }
    ))
}
*/
