use sycamore::{
    futures::spawn_local_scoped,
    reactive::{create_selector, create_signal, Scope, Signal},
    view,
    view::View,
    web::Html,
};
use sycamore_router::navigate;

use crate::ApplicationData;

#[derive(Debug, Clone, Copy)]
enum LoginError {
    WrongCredentials,
    ServerProblem,
    EmptyField,
}

const LOGIN_ADDRESS: &'static str =
    const_format::formatcp!("{}/users/login", crate::SERVER_ADDRESS);

impl LoginError {
    pub fn error_title(&self) -> &'static str {
        match self {
            Self::WrongCredentials => "Wrong credentials.",
            Self::ServerProblem => "Problem with server.",
            Self::EmptyField => "Empty field.",
        }
    }

    pub fn error_description(&self) -> &'static str {
        match self {
            Self::WrongCredentials => "Your e-mail address or / and password is not correct.",
            Self::ServerProblem => {
                "This may be temporary issue with the server. Please try again later."
            }
            Self::EmptyField => "Please fill in all the fields.",
        }
    }
}

async fn login_request(username: String, password: String) -> Result<(), LoginError> {
    let request = gloo_net::http::Request::post(LOGIN_ADDRESS)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&serde_json::json!({
                "username": username,
                "password": password
            }))
            .unwrap(),
        );

    match crate::client::execute(request).await {
        Err(e) => {
            log::error!("Post login request failed: {:?}", e);

            Err(LoginError::ServerProblem)
        }
        Ok(response) => match response.status() {
            200 => Ok(()),
            403 => Err(LoginError::WrongCredentials),
            500 => Err(LoginError::ServerProblem),
            other => {
                log::error!("Got unexpected http status code: {:?})", other);

                panic!("Unimplemented status code.");
            }
        },
    }
}

#[allow(non_snake_case)]
pub fn LoginPage<'a, G: Html>(cx: Scope<'a>, data: ApplicationData<'a>) -> View<G> {
    let username: &Signal<String> = create_signal(cx, String::new());
    let password: &Signal<String> = create_signal(cx, String::new());

    let visible_error: &Signal<Option<LoginError>> = create_signal(cx, None);
    let login_process: &Signal<bool> = create_signal(cx, false);
    let login_error: &Signal<Option<LoginError>> = create_signal(cx, None);
    let successful_login: &Signal<bool> = create_signal(cx, false);

    let username_empty = create_selector(cx, move || username.get().is_empty());
    let password_empty = create_selector(cx, move || password.get().is_empty());

    let submit_button = move |_| {
        if *username_empty.get() || *password_empty.get() {
            visible_error.set(Some(LoginError::EmptyField));
        } else {
            visible_error.set(None);
            login_error.set(None);
            login_process.set(true);

            spawn_local_scoped(cx, async move {
                match login_request(
                    username.get().as_ref().to_owned(),
                    password.get().as_ref().to_owned(),
                )
                .await
                {
                    Ok(()) => {
                        successful_login.set(true);
                        crate::client::update_session_info(cx, data.get_username()).await;

                        navigate("/home");
                    }
                    Err(e) => {
                        login_error.set(Some(e));
                    }
                }

                login_process.set(false);
            });
        }
    };

    let has_error = create_selector(cx, move || visible_error.get().is_some());
    let has_login_error = create_selector(cx, move || login_error.get().is_some());

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

                (
                    if !*login_process.get() {
                        view! {
                            cx,
                            button(class="relative bg-indigo-300 mt-5 w-40 h-10 hover:bg-indigo-500", on:click=submit_button) {
                                "Login"
                            }
                        }
                    } else {
                        view! {
                            cx,
                            button(class="relative bg-indigo-300 mt-5 w-40 h-10 hover:bg-indigo-500") {
                                "Login request..."
                            }
                        }
                    }
                )

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

                (
                    if *has_login_error.get() {
                        view! {
                            cx,
                            div(class="rounded-lg border-2 border-rose-500/75 mt-10 bg-rose-200") {
                                p(class="px-2 text-xl subpixel-antialiased font-extrabold text-center") {
                                    (login_error.get().as_ref().as_ref().unwrap().error_title())
                                }

                                a(class="container px-2 text-m subpixel-antialiased") {
                                    (login_error.get().as_ref().as_ref().unwrap().error_description())
                                }
                            }
                        }
                    } else {
                        view! {
                            cx,

                        }
                    }
                )

                (
                    if *successful_login.get() {
                        view! {
                            cx,
                            div(class="rounded-lg border-2 border-green-500/75 mt-10 bg-green-200") {
                                p(class="px-2 text-xl subpixel-antialiased font-extrabold text-center") {
                                    "Account lo"
                                }

                                a(class="container px-2 text-m subpixel-antialiased") {
                                    "Please login into Your account now."
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
