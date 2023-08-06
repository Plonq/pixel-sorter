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
    pub lower_threshold: u8,
    pub upper_threshold: u8,
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
        let sorted = load_and_sort_img_to_b64(&msg);
        let output = Self::Output { img_data: sorted };

        self.link.respond(id, output)
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}

fn load_and_sort_img_to_b64(input: &WorkerInput) -> Vec<u8> {
    info!("Decoding image");
    let img = image::load_from_memory(input.img_data.as_slice()).unwrap();
    info!("Sorting pixels");
    let img = img::sort_img(img, input.lower_threshold, input.upper_threshold);
    let mut buf: BufWriter<Cursor<Vec<u8>>> = BufWriter::new(Cursor::new(vec![]));
    // This takes the longest (especially if Png)
    info!("Encoding image");
    img.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
    buf.flush().unwrap();
    info!("Done");
    buf.get_ref().to_owned().into_inner()
}
