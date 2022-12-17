use sycamore::{reactive::{Scope, Signal, create_signal, create_selector}, view, view::View, web::Html};

#[derive(Debug, Clone, Copy)]
enum SignupError {
    UsernameTaken,
    EmailTaken,
    PasswordRange,
    FieldMissing
}

impl SignupError {
    pub fn error_title(&self) -> &'static str {
        match self {
            Self::UsernameTaken => {
                "Username already taken."
            },
            Self::EmailTaken => {
                "Email already taken."
            },
            Self::PasswordRange => {
                "Password has invalid length."
            },
            Self::FieldMissing => {
                "One of form's field is missing."
            }
        }
    }

    pub fn error_description(&self) -> &'static str {
        match self {
            Self::UsernameTaken => {
                "Please choose other username."
            },
            Self::EmailTaken => {
                "Found account on given email."
            },
            Self::PasswordRange => {
                "Password has invalid length. It should contain more than 7 characters."
            },
            Self::FieldMissing => {
                "Please fill in all of the form's fields."
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn SignupPage<G: Html>(cx: Scope) -> View<G> {
    let username: &Signal<String> = create_signal(cx, String::new());
    let email: &Signal<String> = create_signal(cx, String::new());
    let password: &Signal<String> = create_signal(cx, String::new());

    let visible_error: &Signal<Option<SignupError>> = create_signal(cx, None);

    let password_length = create_selector(cx, move || password.get().len());
    let username_empty = create_selector(cx, move || username.get().is_empty());
    let email_empty = create_selector(cx, move || email.get().is_empty());

    let submit_button = move |_| {
        let password_length = *password_length.get();

        if password_length == 0 || *username_empty.get() || *email_empty.get() {
            visible_error.set(Some(SignupError::FieldMissing));
        }

        if password_length < 8 {
            visible_error.set(Some(SignupError::PasswordRange));
        }

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

                    label(class="w-full mt-4", for="email") {
                        "E-mail"
                    }

                    br() {}

                    input(
                        class="placeholder:italic rounded-md shadow-inner p-3 mt-2 mb-4",
                        type="text",
                        size="30",
                        name="email",
                        placeholder="E-mail",
                        id="email",
                        bind:value=email) {
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
                    "Register"
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