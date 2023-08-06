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

use crate::agent::{Worker, WorkerInput, WorkerOutput};
use crate::components::Header;

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
    // Worker
    RunWorker,
    WorkerMsg(WorkerOutput),
}

pub struct App {
    // Image
    img: Option<ImageDetails>,
    img_reader: Option<FileReader>,
    loading: bool,
    lower_threshold: u8,
    upper_threshold: u8,
    // Worker
    worker: Box<dyn Bridge<Worker>>,
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
            worker,
            lower_threshold: 50,
            upper_threshold: 200,
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
                true
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
                true
            }
            Msg::SetLowerThreshold(value) => {
                self.lower_threshold = value;
                if self.upper_threshold <= self.lower_threshold {
                    self.upper_threshold = self.lower_threshold;
                }
                true
            }
            Msg::SetUpperThreshold(value) => {
                self.upper_threshold = value;
                if self.lower_threshold >= self.upper_threshold {
                    self.lower_threshold = self.upper_threshold;
                }
                true
            }
            // Worker
            Msg::RunWorker => {
                if let Some(img_details) = &self.img {
                    self.loading = true;
                    self.worker.send(WorkerInput {
                        img_data: img_details.data.clone(),
                        lower_threshold: self.lower_threshold,
                        upper_threshold: self.upper_threshold,
                    });
                }
                true
            }
            Msg::WorkerMsg(output) => {
                // the worker is done!
                if let Some(img) = &mut self.img {
                    img.sorted_data = Some(output.img_data);
                    self.loading = false;
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <Header/>
                <main class="main">
                    <div class={classes!("controls-container")}>
                        <div class={classes!("controls")}>
                            <div
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
                                <Icon icon_id={IconId::FeatherUpload} />
                                <p>{"Drop your image here or click to select"}</p>
                            </div>
                            <input
                                id="file-upload"
                                type="file"
                                accept="image/*"
                                onchange={ctx.link().callback(move |e: Event| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    Self::load_image(input.files())
                                })}
                            />
                            <div class={classes!("threshold")}>
                                <label>
                                    { "Lower threshold: "}
                                    <input
                                        id="lower-threshold"
                                        type="range"
                                        min="0"
                                        max="255"
                                        value={self.lower_threshold.to_string()}
                                        oninput={ctx.link().callback(|e: InputEvent| {
                                            Msg::SetLowerThreshold(e.target_unchecked_into::<HtmlInputElement>().value().parse::<u8>().unwrap())
                                        })}
                                    />
                                    { self.lower_threshold }
                                </label>
                                <label>
                                    { "Upper threshold: "}
                                    <input
                                        id="upper-threshold"
                                        type="range"
                                        min="0"
                                        max="255"
                                        value={self.upper_threshold.to_string()}
                                        oninput={ctx.link().callback(|e: InputEvent| {
                                            Msg::SetUpperThreshold(e.target_unchecked_into::<HtmlInputElement>().value().parse::<u8>().unwrap())
                                        })}
                                    />
                                    { self.upper_threshold }
                                </label>
                            </div>
                            <div class={classes!("button-row")}>
                                <button type="button" onclick={ctx.link().callback(|_| Msg::LoadImage(None))}>{"Reset"}</button>
                                <button type="button" onclick={ctx.link().callback(|_| Msg::RunWorker)}>{"Sort!"}</button>
                            </div>
                        </div>
                    </div>
                    <div class={classes!("output")}>
                        if self.loading {
                            <div>{"Loading..."}</div>
                        } else {
                            if let Some(img_details) = &self.img {
                                { Self::view_img(img_details) }
                            } else {
                                <div class={classes!("placeholder")}>{"Open an image to get started"}</div>
                            }
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
