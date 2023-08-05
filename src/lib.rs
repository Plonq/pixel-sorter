#![recursion_limit = "1024"]
#![allow(clippy::large_enum_variant)]

use std::rc::Rc;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::agent::{Worker, WorkerInput, WorkerOutput};
use crate::components::Header;

pub mod agent;
mod components;

pub enum Msg {
    SetImageB64(String),
    // temp
    Click,
    RunWorker,
    WorkerMsg(WorkerOutput),
}

pub struct App {
    img_b64: String,
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
            img_b64: "".to_string(),
            clicker_value: 0,
            input_ref: NodeRef::default(),
            worker,
            fibonacci_output: String::from("Try out some fibonacci calculations!"),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
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
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html! {
            <>
                <Header/>
                <main class="main">
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
                </main>
                <footer class="footer">
                    { "Powered by Rust, WebAssembly, and the Yew framework. " }
                    <a href="https://github.com/Plonq/recoder">{ "GitHub Repo" }</a>
                    { "." }
                </footer>
            </>
        }
    }
}
