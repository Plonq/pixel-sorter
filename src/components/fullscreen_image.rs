use crate::ImageDetails;
use base64::engine::{general_purpose::STANDARD as b64, Engine};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub img_details: ImageDetails,
    pub onclose: Callback<()>,
}

#[function_component(FullscreenImage)]
pub fn fullscreen_image(props: &Props) -> Html {
    let onclose = props.onclose.clone();
    let (data, file_type) = if let Some(sorted) = &props.img_details.sorted_data {
        // Sorted image is always jpeg (png encoding is really slow)
        (sorted, "image/jpeg".to_string())
    } else {
        (&props.img_details.data, props.img_details.file_type.clone())
    };

    html! {
        <div class="fullscreen-overlay" onclick={Callback::from(move |_| onclose.emit(()))}>
            <img
                src={format!("data:{};base64,{}", file_type, b64.encode(data.as_slice()))}
                alt={props.img_details.name.clone()}
            />
        </div>
    }
}
