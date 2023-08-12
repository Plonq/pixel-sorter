use std::io::{BufWriter, Cursor, Write};

use image::ImageFormat;
use serde::{Deserialize, Serialize};
use yew_agent::{HandlerId, Public, WorkerLink};

use crate::img;
use crate::img::get_orientation;

pub struct Worker {
    link: WorkerLink<Self>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerInput {
    pub img_data: Vec<u8>,
    pub settings: img::SortSettings,
}

#[derive(Serialize, Deserialize)]
pub enum WorkerStatus {
    Decoding,
    Sorting,
    Encoding,
}

#[derive(Serialize, Deserialize)]
pub enum WorkerOutput {
    StatusUpdate(WorkerStatus),
    Result(Vec<u8>),
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
        self.link
            .respond(id, WorkerOutput::StatusUpdate(WorkerStatus::Decoding));
        let mut img = image::load_from_memory(msg.img_data.as_slice()).unwrap();
        if let Some(orientation) = get_orientation(&msg.img_data) {
            img = match orientation {
                3 => img.rotate180(),
                6 => img.rotate90(),
                8 => img.rotate270(),
                _ => img,
            };
        }

        self.link
            .respond(id, WorkerOutput::StatusUpdate(WorkerStatus::Sorting));
        let img = img::sort_img(img, msg.settings.clone());
        let mut buf: BufWriter<Cursor<Vec<u8>>> = BufWriter::new(Cursor::new(vec![]));
        self.link
            .respond(id, WorkerOutput::StatusUpdate(WorkerStatus::Encoding));
        img.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
        buf.flush().unwrap();
        let sorted = buf.get_ref().to_owned().into_inner();
        self.link.respond(id, WorkerOutput::Result(sorted))
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}
