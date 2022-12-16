use dioxus::prelude::*;



#[derive(PartialEq, Props, Default)]
pub struct Account {
    state: LoginState
}

#[derive(PartialEq)]
pub enum LoginState {
    Logged {
        username: String
    },
    NotLogged
}

impl Default for LoginState {
    fn default() -> Self {
        Self::NotLogged
    }
}