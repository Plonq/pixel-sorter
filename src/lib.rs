#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]

use std::rc::Rc;

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
}

pub enum Msg {
    SetImage(Option<File>),
    SetLoading(bool),
    ImageLoaded(String, String, Vec<u8>),
    // temp
    Click,
    RunWorker,
    WorkerMsg(WorkerOutput),
    // img
}

pub struct App {
    img: Option<ImageDetails>,
    img_reader: Option<FileReader>,
    loading: bool,
    // temp
    clicker_value: u32,
    input_ref: NodeRef,
    worker: Box<dyn Bridge<Worker>>,
    fibonacci_output: String,
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
            // demo
            clicker_value: 0,
            input_ref: NodeRef::default(),
            worker,
            fibonacci_output: String::from("Try out some fibonacci calculations!"),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetImage(file) => {
                if let Some(file) = file {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    let link = ctx.link().clone();
                    let file_name = file_name.clone();

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
            Msg::SetLoading(loading) => {
                self.loading = loading;
            }
            Msg::ImageLoaded(file_name, file_type, data) => {
                self.img = Some(ImageDetails {
                    data,
                    file_type,
                    name: file_name,
                });
                self.img_reader = None;
            }
            Self::Message::Click => {
                self.clicker_value += 1;
            }
            Self::Message::RunWorker => {
                if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
                    // start the worker off!
                    self.worker.send(WorkerInput {
                        n: input.value_as_number() as u32,
                    });
                }
            }
            Self::Message::WorkerMsg(output) => {
                // the worker is done!
                self.fibonacci_output = format!("Fibonacci value: {}", output.value);
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
                            <p>{"Open an image to sort its pixels!"}</p>
                            <div
                                id="drop-container"
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
                                <p>{"Drop your images here or click to select"}</p>
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
                            <div class={classes!("button-row")}>
                                <button type="button">{"Reset"}</button>
                                <button type="button">{"Sort!"}</button>
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




                    <div style="display: none">
                        <h1>{ "Web worker demo" }</h1>
                        <p>{ "Submit a value to calculate, then increase the counter on the main thread!"} </p>
                        <p>{ "Large numbers will take some time!" }</p>
                        <h3>{ "Output: " } { &self.fibonacci_output }</h3>
                        <br />
                        <input ref={self.input_ref.clone()} type="number" value="44" max="50"/>
                        <button onclick={ctx.link().callback(|_| Msg::RunWorker)}>{ "submit" }</button>
                        <br /> <br />
                        <h3>{ "Main thread value: " } { self.clicker_value }</h3>
                        <button onclick={ctx.link().callback(|_| Msg::Click)}>{ "click!" }</button>
                        <p id="title">{ "Upload Your Files To The Cloud" }</p>
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
        html! {
            <img src={format!("data:{};base64,{}", img.file_type, base64::encode(&img.data))} alt={img.name.clone()} />
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
            Msg::SetImage(file)
        } else {
            Msg::SetImage(None)
        }
    }
}
