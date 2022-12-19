use gloo_net::http::{Response, self};
use serde::{Deserialize, __private::de};
use sycamore::{
    futures::spawn_local_scoped,
    reactive::{create_selector, create_signal, Scope, Signal},
    suspense::Suspense,
    view,
    view::View,
    web::Html,
    Prop,
};
use zxcvbn::feedback::{Warning, Suggestion};
use std::fmt::Write;

const SIGNUP_ADDRESS: &'static str = const_format::formatcp!("{}/users/signup", crate::SERVER_ADDRESS);

#[derive(Debug, Clone)]
enum SignupError {
    PasswordRange,
    FieldMissing,
    ServerProblem,
    UnprocessableData,
    FieldTaken { username: bool, email: bool },
    WeakPassword {
        warning: Option<Warning>,
        description: Vec<Suggestion>
    }
}

impl SignupError {
    pub fn error_title(&self) -> String {
        match self {
            Self::PasswordRange => "Password has invalid length.".into(),
            Self::FieldMissing => "One of form's field is missing.".into(),
            Self::ServerProblem => "Server problem.".into(),
            Self::UnprocessableData => "Unprocessable data.".into(),
            Self::FieldTaken { .. } => "Account is already registered at credentials.".into(),
            Self::WeakPassword { warning, .. } => {
                let primary_message = "Weak password.";

                if let Some(warn) = warning {
                    format!("{} {}", primary_message, warn)
                } else {
                    primary_message.into()
                }
            }
        }
    }

    pub fn error_description(&self) -> String {
        match self {
            Self::PasswordRange => {
                "Password has invalid length. It should contain more than 7 characters.".into()
            }
            Self::FieldMissing => "Please fill in all of the form's fields.".into(),
            Self::ServerProblem => "Problem with server connection. Please try again later.".into(),
            Self::UnprocessableData => {
                "Server returned some unprocessable data. Please try again later.".into()
            }
            Self::FieldTaken { username, email } => {
                if *username && *email {
                    "Username and email are already taken.".into()
                } else if *username {
                    "Username is already taken.".into()
                } else {
                    "Email is already taken.".into()
                }
            }
            Self::WeakPassword { description, .. } => {
                let mut message = String::new();

                for suggestion in description.into_iter() {
                    write!(&mut message, "{}\n", suggestion).unwrap();
                }

                message
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum SignupMessage {
    FieldTaken {
        username_taken: bool,
        email_taken: bool,
    },
    FieldInvalid {
        username_invalid: bool,
        email_invalid: bool,
    },
}

impl From<SignupMessage> for SignupError {
    fn from(message: SignupMessage) -> Self {
        match message {
            SignupMessage::FieldTaken {
                username_taken,
                email_taken,
            } => Self::FieldTaken {
                username: username_taken,
                email: email_taken,
            },
            SignupMessage::FieldInvalid { .. } => {
                log::error!("Invalid fields detected {message:?}. Please repair implementation.");

                panic!("Repair implementation.");
            }
        }
    }
}

impl SignupError {
    pub async fn extract_from(response: Response) -> SignupError {
        let body = match response.text().await {
            Ok(body) => body,
            Err(e) => {
                log::error!("Unprocessable data: {:?}", e);
                return SignupError::UnprocessableData;
            }
        };

        let message: SignupMessage = match serde_json::from_str(&body) {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("Unprocessable JSON data: {:?}", e);
                return SignupError::UnprocessableData;
            }
        };

        message.into()
    }
}

async fn signup_request(
    username: String,
    email: String,
    password: String,
) -> Result<(), SignupError> {
    let request = gloo_net::http::Request::post(SIGNUP_ADDRESS)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&serde_json::json!({
            "username": username,
            "email": email,
            "password": password
        })).unwrap());

    match crate::client::execute(request).await {
        Err(e) => {
            log::error!("Post request error: {:?}", e);

            Err(SignupError::ServerProblem)
        }
        Ok(response) => {
            log::error!("GOT RESPONSE1");
            match response.status() {
                201 => Ok(()), // created
                500 => Err(SignupError::ServerProblem), // internal server error
                422 | 409 => { // unprocessible entity or conflict
                    Err(SignupError::extract_from(response).await)
                }
                status => {
                    log::error!("Got unrecognized status code. {status:?}");

                    panic!("Critical error.");
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn SignupPage<G: Html>(cx: Scope) -> View<G> {
    let username: &Signal<String> = create_signal(cx, String::new());
    let email: &Signal<String> = create_signal(cx, String::new());
    let password: &Signal<String> = create_signal(cx, String::new());

    let registration_process: &Signal<bool> = create_signal(cx, false);
    let registration_error: &Signal<Option<SignupError>> = create_signal(cx, None);

    let account_creation_successful: &Signal<bool> = create_signal(cx, false);

    let visible_error: &Signal<Option<SignupError>> = create_signal(cx, None);

    let password_length = create_selector(cx, move || password.get().len());
    let username_empty = create_selector(cx, move || username.get().is_empty());
    let email_empty = create_selector(cx, move || email.get().is_empty());

    let submit_button = move |_| {
        account_creation_successful.set(false);
        let password_length = *password_length.get();

        if password_length == 0 || *username_empty.get() || *email_empty.get() {
            visible_error.set(Some(SignupError::FieldMissing));
        } else if password_length < 8 {
            visible_error.set(Some(SignupError::PasswordRange));
        } else {
            let password_grade = zxcvbn::zxcvbn(
                password.get().as_str(),
                &[
                    username.get().as_str(),
                    email.get().as_str()
                ]).unwrap();
            
            if password_grade.score() < 3 {
                // Password is too weak.
                let feedback = password_grade.feedback().as_ref().unwrap();
                let warning = feedback.warning();
                let description = feedback.suggestions().to_owned();

                visible_error.set(Some(SignupError::WeakPassword { warning, description }));
            } else {
                visible_error.set(None);
                registration_process.set(true);
                spawn_local_scoped(cx, async move {
                    match signup_request(
                        username.get().as_str().to_string(),
                        email.get().as_str().to_string(),
                        password.get().as_str().to_string(),
                    )
                    .await
                    {
                        Ok(()) => {
                            account_creation_successful.set(true);
                        }
                        Err(e) => {
                            registration_error.set(Some(e));
                        }
                    }
    
                    registration_process.set(false);
                });
            }
        }
    };

    let has_error = create_selector(cx, move || visible_error.get().is_some());
    let has_registration_error = create_selector(cx, move || registration_error.get().is_some());

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

                (
                    if !*registration_process.get() {
                        view! {
                            cx,
                            button(class="relative bg-indigo-300 mt-5 w-40 h-10 hover:bg-indigo-500", on:click=submit_button) {
                                "Register"
                            }
                        }
                    } else {
                        view! {
                            cx,
                            button(class="relative bg-indigo-300 mt-5 w-40 h-10 hover:bg-indigo-500") {
                                "Registering..."
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
                                    (visible_error.get().as_ref().as_ref().unwrap().error_title())
                                }

                                a(class="container px-2 text-m subpixel-antialiased") {
                                    (visible_error.get().as_ref().as_ref().unwrap().error_description())
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
                    if *has_registration_error.get() {
                        view! {
                            cx,
                            div(class="rounded-lg border-2 border-rose-500/75 mt-10 bg-rose-200") {
                                p(class="px-2 text-xl subpixel-antialiased font-extrabold text-center") {
                                    (registration_error.get().as_ref().as_ref().unwrap().error_title())
                                }

                                a(class="container px-2 text-m subpixel-antialiased") {
                                    (registration_error.get().as_ref().as_ref().unwrap().error_description())
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
                    if *account_creation_successful.get() {
                        view! {
                            cx,
                            div(class="rounded-lg border-2 border-green-500/75 mt-10 bg-green-200") {
                                p(class="px-2 text-xl subpixel-antialiased font-extrabold text-center") {
                                    "Account created successfully!"
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
