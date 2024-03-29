use yew::prelude::*;

/// App header
#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <>
            <header>
                <h1 class="sr-only">{ "Pixel Sorter" }</h1>
                <img width="300" height="55" src="assets/pixel-sorter-logo.png" alt="Pixel Sorter logo" />
            </header>
            <p class="site-description">
                { "Sort pixels to get glitchy effects! No uploads, runs in browser." }
            </p>
        </>
    }
}
