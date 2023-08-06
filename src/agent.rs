use image::ImageFormat;
use log::info;
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Cursor, Write};
use yew_agent::{HandlerId, Public, WorkerLink};

use crate::img;

pub struct Worker {
    link: WorkerLink<Self>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerInput {
    pub img_data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerOutput {
    pub img_data: Vec<u8>,
}

impl yew_agent::Worker for Worker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = WorkerInput;
    type Output = WorkerOutput;

    fn create(link: WorkerLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) {
        // no messaging
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        info!("Hi from worker!");
        // this runs in a web worker
        // and does not block the main
        // browser thread!

        let img_data = msg.img_data;
        let sorted = load_and_sort_img_to_b64(&img_data);
        let output = Self::Output { img_data: sorted };

        self.link.respond(id, output)

        // let n = msg.n;
        //
        // fn fib(n: u32) -> u32 {
        //     if n <= 1 {
        //         1
        //     } else {
        //         fib(n - 1) + fib(n - 2)
        //     }
        // }
        //
        // let output = Self::Output { value: fib(n) };
        //
        // self.link.respond(id, output);
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}

fn load_and_sort_img_to_b64(data: &Vec<u8>) -> Vec<u8> {
    let img = image::load_from_memory(data.as_slice()).unwrap();
    let img = img::sort_img(img);
    let mut buf: BufWriter<Cursor<Vec<u8>>> = BufWriter::new(Cursor::new(vec![]));
    // This takes the longest (especially if Png)
    img.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
    buf.flush().unwrap();
    buf.get_ref().to_owned().into_inner()
}
