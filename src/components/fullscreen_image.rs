use base64::engine::{general_purpose::STANDARD as b64, Engine};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
    #[prop_or("image/jpeg".to_string())]
    pub file_type: String,
    pub data: Vec<u8>,
    pub onclose: Callback<()>,
}

#[function_component(FullscreenImage)]
pub fn fullscreen_image(props: &Props) -> Html {
    let onclose = props.onclose.clone();

    html! {
        <div class="fullscreen-overlay" onclick={Callback::from(move |_| onclose.emit(()))}>
            <img
                src={format!("data:{};base64,{}", props.file_type, b64.encode(props.data.as_slice()))}
                alt={props.name.clone()}
            />
        </div>
    }
}
