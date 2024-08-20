use yew::prelude::*;
use gloo_utils::{body, document};
use web_sys::*;
use wasm_bindgen::prelude::*;

pub enum Msg {
    ToggleDarkMode
}

pub struct NavBar;

impl Component for NavBar {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }
    
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleDarkMode => {
                let darkmode_text = document().get_element_by_id("darkmode-text").unwrap_throw();
                let darkmode_icon = document().get_element_by_id("darkmode-icon").unwrap_throw();
                let logo = document().get_element_by_id("logo").unwrap_throw().unchecked_into::<HtmlElement>();
                    
                match body().class_list().toggle("dark-mode").unwrap_throw() {
                    true => {
                        darkmode_text.set_text_content(Some("Modo claro"));
                        darkmode_icon.set_attribute("name", "sunny-outline").unwrap_throw();
                        logo.style().set_property("filter", "invert(1)").unwrap_throw();
                    },
                    false => {
                        darkmode_text.set_text_content(Some("Modo oscuro"));
                        darkmode_icon.set_attribute("name", "moon-outline").unwrap_throw();
                        logo.style().set_property("filter", "invert(0)").unwrap_throw();
                    }
                };

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <nav class="navbar">
                <ul>
                    <li class="logo">
                        <div class="content">
                            <img src="assets/soluai-logo.svg" alt="SoluAI logo" id="logo"/>
                        </div>
                    </li>
                    <li>
                        <button class="active">
                            <ion-icon name="videocam-outline"></ion-icon>
                            <span class="text">{ "Cámaras" }</span>
                        </button>
                    </li>
                    <li>
                        <button>
                            <ion-icon name="cube-outline"></ion-icon>
                            <span class="text">{ "Modelos IA" }</span>
                        </button>
                    </li>
                    <li>
                        <button>
                            <ion-icon name="search-outline"></ion-icon>
                            <span class="text">{ "Búsqueda general" }</span>
                        </button>
                    </li>
                    <li>
                        <button class="notifications">
                            <ion-icon name="notifications-outline"></ion-icon>
                            <div class="text">
                                <span>{ "Notificaciones" }</span>
                                <span class="number">{ "4" }</span>
                            </div>
                        </button>
                    </li>
                    <li>
                        <button>
                            <ion-icon name="bookmark-outline"></ion-icon>
                            <span class="text">{ "Capturas" }</span>
                        </button>
                    </li>
                    <li>
                        <button>
                            <ion-icon name="cog-outline"></ion-icon>
                            <span class="text">{ "Configuración" }</span>
                        </button>
                    </li>
                    <li>
                        <button>
                            <ion-icon name="help-circle-outline"></ion-icon>
                            <span class="text">{ "Ayuda" }</span>
                        </button>
                    </li>
                    <li>
                        <button class="darkmode-button" onclick={ ctx.link().callback(|_| Msg::ToggleDarkMode) }>
                            <ion-icon id="darkmode-icon" name="moon-outline"></ion-icon>
                            <span id="darkmode-text" class="text">{ "Modo oscuro" }</span>
                        </button>
                    </li>
                    <li class="user">
                        <button>
                            <div class="content">
                                <img src="assets/soluai-user.png" alt="User"/>
                            </div>
                            <div class="text title">
                                <h1>{ "SoluAI" }</h1>
                                <p>{ "Bienvenido" }</p>
                            </div>
                        </button>
                    </li>
                </ul>
            </nav>
        }
    }
    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let nav_buttons_query = document().query_selector_all(".navbar ul li button:not(.darkmode-button)").unwrap_throw();
            let nav_buttons_list = (0..nav_buttons_query.length()).map(|i| {
                nav_buttons_query.item(i).unwrap_throw().unchecked_into::<HtmlElement>()
            });
            
            let closure = Closure::wrap(Box::new(|e: Event| {
                let nav_button = e
                    .current_target()
                    .unwrap_throw()
                    .unchecked_into::<HtmlElement>();
    
                let nav_buttons_query = document().query_selector_all(".navbar ul li button").unwrap_throw();
                let nav_buttons_list = (0..nav_buttons_query.length()).map(|i| {
                    nav_buttons_query.item(i).unwrap_throw().unchecked_into::<HtmlElement>()
                });
                for nav_button in nav_buttons_list {
                    nav_button.class_list().remove_1("active").unwrap_throw();
                } 
                
                nav_button.class_list().add_1("active").unwrap_throw();
            }) as Box<dyn FnMut(_)>);
            
    
            for nav_button in nav_buttons_list {
                nav_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap_throw();
            }
            
            closure.forget();
        }
    }
}