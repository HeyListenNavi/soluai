mod app;
mod pages;
mod components;
use yew::Renderer;
use app::App;

fn main() {
    Renderer::<App>::new().render();
}