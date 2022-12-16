use dioxus::prelude::*;


pub fn login_page(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            class: "h-screen w-screen bg-indigo-200 items-center justify-center",
            div {
                class: "columns-1 flex-col flex flex-wrap items-center justify-center",
                
                form {
                    label {
                        class: "w-full",
                        r#for: "username",
                        "Username"
                    }
                    br {}
                    input {
                        class: "placeholder:italic",
                        r#type: "text",
                        name: "username",
                        placeholder: "Username",
                        id: "username"
                    }
                    br {}
                    label {
                        r#for: "email",
                        "E-mail"
                    }
                    br {}
                    input {
                        class: "placeholder:italic",
                        r#type: "text",
                        name: "email",
                        placeholder: "E-mail",
                        id: "email"
                    }
                    br {}
                    label {
                        r#for: "password",
                        "Password"
                    }
                    br {
                        class: "h-15"
                    }
                    input {
                        class: "placeholder:italic",
                        r#type: "password",
                        name: "password",
                        placeholder: "Password",
                        id: "password"
                    }
                }
            }
        }
    ))
}