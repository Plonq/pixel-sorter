#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]

use std::collections::HashMap;
use std::rc::Rc;

use gloo::file::callbacks::FileReader;
use gloo::file::File;
use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::html::TargetCast;
use yew::prelude::*;
use yew::{html, Callback, Component, Context, Html};
use yew_agent::{Bridge, Bridged};

use crate::agent::{Worker, WorkerInput, WorkerOutput};
use crate::components::Header;

pub mod agent;
mod components;
mod img;

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

pub enum Msg {
    SetImageB64(String),
    // temp
    Click,
    RunWorker,
    WorkerMsg(WorkerOutput),
    // img
    Loaded(String, String, Vec<u8>),
    Files(Vec<File>),
}

pub struct App {
    img_b64: String,
    // temp
    clicker_value: u32,
    input_ref: NodeRef,
    worker: Box<dyn Bridge<Worker>>,
    fibonacci_output: String,
    // img
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
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
            img_b64: "".to_string(),
            clicker_value: 0,
            input_ref: NodeRef::default(),
            worker,
            fibonacci_output: String::from("Try out some fibonacci calculations!"),
            readers: HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetImageB64(_) => (),
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
            Msg::Loaded(file_name, file_type, data) => {
                self.files.push(FileDetails {
                    data,
                    file_type,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(
                                file_name,
                                file_type,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
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
                    <label for="file-upload">
                        <div
                            id="drop-container"
                            ondrop={ctx.link().callback(|event: DragEvent| {
                                event.prevent_default();
                                let files = event.data_transfer().unwrap().files();
                                Self::upload_files(files)
                            })}
                            ondragover={Callback::from(|event: DragEvent| {
                                event.prevent_default();
                            })}
                            ondragenter={Callback::from(|event: DragEvent| {
                                event.prevent_default();
                            })}
                        >
                            <i class="fa fa-cloud-upload"></i>
                            <p>{"Drop your images here or click to select"}</p>
                        </div>
                    </label>
                    <input
                        id="file-upload"
                        type="file"
                        accept="image/*"
                        multiple={true}
                        onchange={ctx.link().callback(move |e: Event| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Self::upload_files(input.files())
                        })}
                    />
                    <div id="preview-area">
                        { for self.files.iter().map(Self::view_file) }
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
    fn view_file(file: &FileDetails) -> Html {
        html! {
            <div class="preview-tile">
                <p class="preview-name">{ format!("{}", file.name) }</p>
                <div class="preview-media">
                    if file.file_type.contains("image") {
                        <img src={format!("data:{};base64,{}", file.file_type, base64::encode(&file.data))} />
                    } else if file.file_type.contains("video") {
                        <video controls={true}>
                            <source src={format!("data:{};base64,{}", file.file_type, base64::encode(&file.data))} type={file.file_type.clone()}/>
                        </video>
                    }
                </div>
            </div>
        }
    }

    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}
