use yew::prelude::*;

#[derive(PartialEq, Default)]
pub enum Style {
    #[default]
    Standard,
    // Primary,
    // Borderless,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub style: Style,
    #[prop_or(false)]
    pub submit: bool,
    pub onclick: Callback<MouseEvent>,
    #[prop_or(false)]
    pub disabled: bool,
    pub children: Children,
}

#[function_component(Button)]
pub fn button(props: &Props) -> Html {
    let onclick = props.onclick.clone();
    let disabled = props.disabled;

    let class = vec!["btn"];
    // class.push(match props.style {
    //     Style::Primary => "primary",
    //     Style::Borderless => "borderless",
    //     _ => "",
    // });

    let button_type = if props.submit { "submit" } else { "button" };

    html! {
        <button
            type={button_type}
            class={classes!(class)}
            {disabled}
            onclick={Callback::from(move |event: MouseEvent| onclick.emit(event.clone()))}
        >
            { for props.children.iter() }
        </button>
    }
}
