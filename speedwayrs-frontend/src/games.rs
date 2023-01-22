use sycamore::reactive::Signal;
use sycamore::{view, web::Html, reactive::Scope, view::View};
use sycamore::Prop;

#[derive(Prop)]
pub struct GameInfoProp<'a> {
    username: &'a Signal<Option<String>>
}

pub fn GamesPage<'a, G: Html>(cx: Scope<'a>, info: GameInfoProp<'a>) -> View<G> {
    view! {
        cx,
        div(class="h-full w-full grid grid-rows-2 grid-cols-2") {

        }
    }
}
