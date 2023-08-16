use std::rc::Rc;

use base64::engine::{general_purpose::STANDARD as b64, Engine};
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::html::TargetCast;
use yew::prelude::*;
use yew::{html, Callback, Component, Context, Html};
use yew_agent::{Bridge, Bridged};
use yew_icons::{Icon, IconId};

use crate::agent::{Worker, WorkerInput, WorkerOutput, WorkerStatus};
use crate::components::{Button, ButtonStyle, FullscreenImage, Header};
use crate::img::{Direction, Order, SortSettings};

pub mod agent;
mod components;
mod img;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageDetails {
    pub name: String,
    pub file_type: String,
    pub data: Vec<u8>,
}

pub enum Msg {
    // Image
    LoadImage(Option<File>),
    ImageLoaded(String, String, Vec<u8>),
    SetLowerThreshold(u8),
    SetUpperThreshold(u8),
    SetDirection(Direction),
    SetOrder(Order),
    SettingsChanged,
    ToggleShowOriginal,
    Reset,
    ClearImage,
    ToggleZoom,
    // Worker
    RunWorker,
    WorkerMsg(WorkerOutput),
}

pub struct App {
    // Image
    img: Option<ImageDetails>,
    img_reader: Option<FileReader>,
    sort_settings: SortSettings,
    zoomed: bool,
    sorted_data: Option<Vec<u8>>,
    show_original: bool,
    // Worker
    worker: Box<dyn Bridge<Worker>>,
    worker_status: Option<WorkerStatus>,
}

// todo:
// - Make it easier to replace image (esp on mobile) without resetting settings. Make resetting settings not clear image, and re-run mask/sort.
impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let cb = {
            let link = ctx.link().clone();
            move |e| link.send_message(Self::Message::WorkerMsg(e))
        };
        let worker = Worker::bridge(Rc::new(cb));

        Self {
            img: None,
            img_reader: None,
            sort_settings: SortSettings::default(),
            zoomed: false,
            sorted_data: None,
            show_original: false,
            worker,
            worker_status: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadImage(file) => {
                self.sorted_data = None;
                if let Some(file) = file {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    let link = ctx.link().clone();
                    let file_name = file_name;

                    self.img_reader =
                        Some(gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::ImageLoaded(
                                file_name,
                                file_type,
                                res.expect("failed to read file"),
                            ))
                        }));
                } else {
                    self.img = None;
                    self.img_reader = None;
                }
            }
            Msg::ImageLoaded(file_name, file_type, data) => {
                self.img = Some(ImageDetails {
                    data,
                    file_type,
                    name: file_name,
                });
                self.img_reader = None;
                ctx.link().send_message(Msg::RunWorker);
            }
            Msg::SetLowerThreshold(value) => {
                self.sort_settings.lower_threshold = value;
                if self.sort_settings.upper_threshold <= self.sort_settings.lower_threshold {
                    self.sort_settings.upper_threshold = self.sort_settings.lower_threshold;
                }
                ctx.link().send_message(Msg::SettingsChanged)
            }
            Msg::SetUpperThreshold(value) => {
                self.sort_settings.upper_threshold = value;
                if self.sort_settings.lower_threshold >= self.sort_settings.upper_threshold {
                    self.sort_settings.lower_threshold = self.sort_settings.upper_threshold;
                }
                ctx.link().send_message(Msg::SettingsChanged)
            }
            Msg::SetDirection(direction) => {
                self.sort_settings.direction = direction;
                ctx.link().send_message(Msg::SettingsChanged)
            }
            Msg::SetOrder(order) => {
                self.sort_settings.order = order;
                ctx.link().send_message(Msg::SettingsChanged)
            }
            Msg::Reset => {
                self.sort_settings = SortSettings::default();
                ctx.link().send_message(Msg::RunWorker);
            }
            Msg::ClearImage => {
                self.img = None;
            }
            Msg::ToggleZoom => {
                self.zoomed = !self.zoomed;
            }
            Msg::SettingsChanged => ctx.link().send_message(Msg::RunWorker),
            Msg::ToggleShowOriginal => {
                self.show_original = !self.show_original;
            }
            // Worker
            Msg::RunWorker => {
                if let Some(img_details) = &self.img {
                    self.worker.send(WorkerInput {
                        img_data: img_details.data.clone(),
                        settings: self.sort_settings.clone(),
                    });
                }
            }
            Msg::WorkerMsg(output) => {
                // the worker is done!
                match output {
                    WorkerOutput::StatusUpdate(status) => {
                        self.worker_status = Some(status);
                    }
                    WorkerOutput::Sorted(img_data) => {
                        self.sorted_data = Some(img_data);
                        self.worker_status = None;
                    }
                }
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <Header/>
                <main class="main">
                    <div class={classes!("controls-container")}>
                        <div class={classes!("controls")}>
                            <fieldset class={classes!("threshold")}>
                                <legend title="This mask determines which pixels will be sorted. Pixels brighter than the lower threshold and darker than the upper threshold will be sorted.">{ "Image mask" }</legend>
                                <div class="threshold-grid">
                                    <label for="lower-threshold">{ "Lower threshold: "}</label>
                                    <input
                                        id="lower-threshold"
                                        type="range"
                                        min="0"
                                        max="255"
                                        value={self.sort_settings.lower_threshold.to_string()}
                                        onchange={ctx.link().callback(|e: Event| {
                                            Msg::SetLowerThreshold(e.target_unchecked_into::<HtmlInputElement>().value().parse::<u8>().unwrap())
                                        })}
                                    />
                                    <span>{ self.sort_settings.lower_threshold }</span>
                                    <label for="upper-threshold">{ "Upper threshold: "}</label>
                                    <input
                                        id="upper-threshold"
                                        type="range"
                                        min="0"
                                        max="255"
                                        value={self.sort_settings.upper_threshold.to_string()}
                                        onchange={ctx.link().callback(|e: Event| {
                                            Msg::SetUpperThreshold(e.target_unchecked_into::<HtmlInputElement>().value().parse::<u8>().unwrap())
                                        })}
                                    />
                                    <span>{ self.sort_settings.upper_threshold }</span>
                                </div>
                            </fieldset>
                            <fieldset class={classes!("direction")}>
                                <legend>{ "Sort Direction" }</legend>
                                <div class="custom-radio-group">
                                    <label class="custom-radio">
                                        <input
                                            type="radio"
                                            checked={self.sort_settings.direction == img::Direction::Horizontal}
                                            onchange={ctx.link().callback(|_: Event| Msg::SetDirection(img::Direction::Horizontal))}
                                        />
                                        <span>{"Horizontal"}</span>
                                    </label>
                                    <label class="custom-radio">
                                        <input
                                            type="radio"
                                            checked={self.sort_settings.direction == img::Direction::Vertical}
                                            onchange={ctx.link().callback(|_: Event| Msg::SetDirection(img::Direction::Vertical))}
                                        />
                                        <span>{"Vertical"}</span>
                                    </label>
                                </div>
                            </fieldset>
                            <fieldset class={classes!("order")}>
                                <legend>{ "Sort Order" }</legend>
                                <div class="custom-radio-group">
                                    <label class="custom-radio">
                                        <input
                                            type="radio"
                                            checked={self.sort_settings.order == img::Order::Ascending}
                                            onchange={ctx.link().callback(|_: Event| Msg::SetOrder(img::Order::Ascending))}
                                        />
                                        <span>{"Ascending"}</span>
                                    </label>
                                    <label class="custom-radio">
                                        <input
                                            type="radio"
                                            checked={self.sort_settings.order == img::Order::Descending}
                                            onchange={ctx.link().callback(|_: Event| Msg::SetOrder(img::Order::Descending))}
                                        />
                                        <span>{"Descending"}</span>
                                    </label>
                                </div>
                            </fieldset>
                            <div class={classes!("button-row")}>
                                if self.img.is_some() {
                                    <label class="custom-checkbox">
                                        <div class="box">
                                            <input
                                                type="checkbox"
                                                checked={self.show_original}
                                                onchange={ctx.link().callback(|_: Event| Msg::ToggleShowOriginal)}
                                            />
                                            <svg class="checkmark" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="currentColor" d="M20.285 2l-11.285 11.567-5.286-5.011-3.714 3.716 9 8.728 15-15.285z"/></svg>
                                        </div>
                                        <span>{ "Show original" }</span>
                                    </label>
                                    <Button
                                        disabled={self.worker_status.is_some()}
                                        onclick={ctx.link().callback(|_| Msg::ClearImage)}
                                    >
                                        { "Clear Image" }
                                    </Button>
                                }
                                <Button
                                    disabled={self.worker_status.is_some()}
                                    onclick={ctx.link().callback(|_| Msg::Reset)}
                                >
                                    { "Reset" }
                                </Button>
                            </div>
                        </div>
                    </div>
                    <div
                        class={classes!("output", "overlay-container", self.img.is_some().then_some(Some("has-image")))}
                        ondrop={ctx.link().callback(|event: DragEvent| {
                            event.prevent_default();
                            let files = event.data_transfer().unwrap().files();
                            Self::load_image(files)
                        })}
                        ondragover={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                        ondragenter={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                    >
                        if let Some(img_details) = &self.img {
                            { self.view_img(ctx, img_details) }
                        } else {
                            <label
                                for="file-upload"
                                class={classes!("placeholder", "drop-container")}
                            >
                                <Icon icon_id={IconId::LucideImagePlus} />
                                <p>{"Drop your image here or click to select"}</p>
                                <input
                                    id="file-upload"
                                    class="sr-only"
                                    type="file"
                                    accept="image/*"
                                    onchange={ctx.link().callback(move |e: Event| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        Self::load_image(input.files())
                                    })}
                                />
                            </label>
                        }
                    </div>
                </main>
                <footer class="footer">
                    { "Powered by Rust, WebAssembly, and the Yew framework. " }
                    <a href="https://github.com/Plonq/pixel-sorter">{ "GitHub Repo" }</a>
                    { "." }
                </footer>
            </>
        }
    }
}

impl App {
    fn view_img(&self, ctx: &Context<Self>, img: &ImageDetails) -> Html {
        let (data, file_type) =
            if let (Some(sorted), false) = (&self.sorted_data, self.show_original) {
                // Sorted image is always jpeg (png encoding is really slow)
                (sorted, "image/jpeg".to_string())
            } else {
                (&img.data, img.file_type.clone())
            };

        html! {
            <>
                <img
                    onclick={ctx.link().callback(move |_| Msg::ToggleZoom)}
                    src={format!("data:{};base64,{}", file_type, b64.encode(data.as_slice()))}
                    alt={img.name.clone()}
                />
                if self.worker_status.is_some() {
                    <div class={classes!("overlay")}>
                        <div class={classes!("content")}>
                            <Icon icon_id={IconId::LucideLoader} />
                            if let Some(status) = &self.worker_status {
                                {match status {
                                    WorkerStatus::Decoding => {"Decoding image"},
                                    WorkerStatus::Sorting => {"Sorting the pixels"},
                                    WorkerStatus::Masking => {"Generating mask"},
                                    WorkerStatus::Encoding => {"Encoding the image"},
                                }}
                            }
                        </div>
                    </div>
                }
                if self.zoomed {
                    <FullscreenImage
                        data={data.clone()}
                        name={img.name.clone()}
                        file_type={file_type}
                        onclose={ctx.link().callback(|_| Msg::ToggleZoom)}
                    />
                }
            </>
        }
    }

    fn load_image(files: Option<FileList>) -> Msg {
        if let Some(files) = files {
            let file = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from)
                .next();
            Msg::LoadImage(file)
        } else {
            Msg::LoadImage(None)
        }
    }
}
