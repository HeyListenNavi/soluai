use yew::prelude::*;
use crate::components::camera_frame::CameraFrame;
use crate::components::header::Header;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <>
            <Header/>
            <CameraFrame/>
        </>
    }
}
