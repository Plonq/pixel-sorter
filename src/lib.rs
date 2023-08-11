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
use crate::components::{Button, ButtonStyle, Header};
use crate::img::{Direction, Order, SortSettings};

pub mod agent;
mod components;
mod img;

struct ImageDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
    sorted_data: Option<Vec<u8>>,
}

pub enum Msg {
    // Image
    LoadImage(Option<File>),
    ImageLoaded(String, String, Vec<u8>),
    SetLowerThreshold(u8),
    SetUpperThreshold(u8),
    SetDirection(Direction),
    SetOrder(Order),
    // Worker
    RunWorker,
    WorkerMsg(WorkerOutput),
}

pub struct App {
    // Image
    img: Option<ImageDetails>,
    img_reader: Option<FileReader>,
    loading: bool,
    sort_settings: SortSettings,
    // Worker
    worker: Box<dyn Bridge<Worker>>,
    worker_status: Option<WorkerStatus>,
}

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
            loading: false,
            sort_settings: SortSettings {
                lower_threshold: 50,
                upper_threshold: 150,
                direction: Direction::Horizontal,
                order: Order::Ascending,
            },
            worker,
            worker_status: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadImage(file) => {
                if let Some(file) = file {
                    self.loading = true;
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
                    sorted_data: None,
                });
                self.img_reader = None;
                self.loading = false;
            }
            Msg::SetLowerThreshold(value) => {
                self.sort_settings.lower_threshold = value;
                if self.sort_settings.upper_threshold <= self.sort_settings.lower_threshold {
                    self.sort_settings.upper_threshold = self.sort_settings.lower_threshold;
                }
            }
            Msg::SetUpperThreshold(value) => {
                self.sort_settings.upper_threshold = value;
                if self.sort_settings.lower_threshold >= self.sort_settings.upper_threshold {
                    self.sort_settings.lower_threshold = self.sort_settings.upper_threshold;
                }
            }
            Msg::SetDirection(direction) => {
                self.sort_settings.direction = direction;
            }
            Msg::SetOrder(order) => {
                self.sort_settings.order = order;
            }
            // Worker
            Msg::RunWorker => {
                if let Some(img_details) = &self.img {
                    self.loading = true;
                    self.worker.send(WorkerInput {
                        img_data: img_details.data.clone(),
                        settings: self.sort_settings.clone(),
                    });
                }
            }
            Msg::WorkerMsg(output) => {
                // the worker is done!
                if let Some(img) = &mut self.img {
                    match output {
                        WorkerOutput::StatusUpdate(status) => {
                            self.worker_status = Some(status);
                        }
                        WorkerOutput::Result(img_data) => {
                            img.sorted_data = Some(img_data);
                            self.loading = false;
                            self.worker_status = None;
                        }
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
                            <label
                                for="file-upload"
                                class={classes!("drop-container")}
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
                            <div class={classes!("threshold")}>
                                <label>
                                    { "Lower threshold: "}
                                    <input
                                        id="lower-threshold"
                                        type="range"
                                        min="0"
                                        max="255"
                                        value={self.sort_settings.lower_threshold.to_string()}
                                        oninput={ctx.link().callback(|e: InputEvent| {
                                            Msg::SetLowerThreshold(e.target_unchecked_into::<HtmlInputElement>().value().parse::<u8>().unwrap())
                                        })}
                                    />
                                    { self.sort_settings.lower_threshold }
                                </label>
                                <label>
                                    { "Upper threshold: "}
                                    <input
                                        id="upper-threshold"
                                        type="range"
                                        min="0"
                                        max="255"
                                        value={self.sort_settings.upper_threshold.to_string()}
                                        oninput={ctx.link().callback(|e: InputEvent| {
                                            Msg::SetUpperThreshold(e.target_unchecked_into::<HtmlInputElement>().value().parse::<u8>().unwrap())
                                        })}
                                    />
                                    { self.sort_settings.upper_threshold }
                                </label>
                            </div>
                            <div class={classes!("direction")}>
                                <label>
                                    <input
                                        type="radio"
                                        checked={self.sort_settings.direction == img::Direction::Horizontal}
                                        onchange={ctx.link().callback(|_: Event| Msg::SetDirection(img::Direction::Horizontal))}
                                    />
                                    {"Horizontal"}
                                </label>
                                <label>
                                    <input
                                        type="radio"
                                        checked={self.sort_settings.direction == img::Direction::Vertical}
                                        onchange={ctx.link().callback(|_: Event| Msg::SetDirection(img::Direction::Vertical))}
                                    />
                                    {"Vertical"}
                                </label>
                            </div>
                            <div class={classes!("order")}>
                                <label>
                                    <input
                                        type="radio"
                                        checked={self.sort_settings.order == img::Order::Ascending}
                                        onchange={ctx.link().callback(|_: Event| Msg::SetOrder(img::Order::Ascending))}
                                    />
                                    {"Ascending"}
                                </label>
                                <label>
                                    <input
                                        type="radio"
                                        checked={self.sort_settings.order == img::Order::Descending}
                                        onchange={ctx.link().callback(|_: Event| Msg::SetOrder(img::Order::Descending))}
                                    />
                                    {"Descending"}
                                </label>
                            </div>
                            <div class={classes!("button-row")}>
                                // <button type="button" class="btn" onclick={ctx.link().callback(|_| Msg::LoadImage(None))}>{"Reset"}</button>
                                <Button
                                    style={ButtonStyle::Primary}
                                    disabled={self.worker_status.is_some() || self.img.is_none()}
                                    onclick={ctx.link().callback(|_| Msg::RunWorker)}
                                >
                                    { "Sort!" }
                                </Button>
                            </div>
                        </div>
                    </div>
                    <div class={classes!("output", "overlay-container")}>
                        if let Some(img_details) = &self.img {
                            { Self::view_img(img_details) }
                        } else {
                            <div class={classes!("placeholder")}>{"Open an image to get started"}</div>
                        }
                        if self.worker_status.is_some() {
                            <div class={classes!("overlay")}>
                                <div class={classes!("content")}>
                                    <Icon icon_id={IconId::LucideLoader} />
                                    if let Some(status) = &self.worker_status {
                                        {match status {
                                            WorkerStatus::Decoding => {"Decoding image..."},
                                            WorkerStatus::Sorting => {"Sorting the pixels..."},
                                            WorkerStatus::Encoding => {"Encoding the image..."},
                                        }}
                                    }
                                </div>
                            </div>
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
    fn view_img(img: &ImageDetails) -> Html {
        let (data, file_type) = if let Some(sorted) = &img.sorted_data {
            // Sorted image is always jpeg (png encoding is really slow)
            (sorted, "image/jpeg".to_string())
        } else {
            (&img.data, img.file_type.clone())
        };
        html! {
            <img src={format!("data:{};base64,{}", file_type, b64.encode(data.as_slice()))} alt={img.name.clone()} />
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
