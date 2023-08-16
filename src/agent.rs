use image::ImageFormat;
use serde::{Deserialize, Serialize};
use yew_agent::{HandlerId, Public, WorkerLink};

use crate::img::{get_orientation, ImageToBytes, sort_img, SortSettings};

pub struct Worker {
    link: WorkerLink<Self>,
}

#[derive(Serialize, Deserialize)]
pub struct WorkerInput {
    pub img_data: Vec<u8>,
    pub settings: SortSettings,
}

#[derive(Serialize, Deserialize)]
pub enum WorkerStatus {
    Decoding,
    Sorting,
    Masking,
    Encoding,
}

#[derive(Serialize, Deserialize)]
pub enum WorkerOutput {
    StatusUpdate(WorkerStatus),
    Sorted(Vec<u8>),
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
        let img = sort_img(img, msg.settings);
        self.link
            .respond(id, WorkerOutput::StatusUpdate(WorkerStatus::Encoding));
        let sorted = img.to_bytes(ImageFormat::Jpeg);
        self.link.respond(id, WorkerOutput::Sorted(sorted))
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}
