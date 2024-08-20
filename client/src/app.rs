use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::home::Home;

use crate::components::nav_bar::NavBar;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home/> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>
            <HashRouter>
                <NavBar/>
                <div class="main">
                    <Switch<Route> render={switch}/>
                </div>
            </HashRouter>
        </>
    }
}