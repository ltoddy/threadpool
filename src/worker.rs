use std::sync::{
    mpsc::Receiver,
    {Arc, Mutex},
};
use std::thread;

use crate::Message;

pub(crate) struct Worker {
    pub(crate) _id: usize,
    pub(crate) thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub(crate) fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || Self::work(receiver));

        Self {
            _id: id,
            thread: Some(thread),
        }
    }

    fn work(receiver: Arc<Mutex<Receiver<Message>>>) {
        loop {
            let message = receiver
                .lock()
                .expect("Poisoned thread")
                .recv()
                .expect("channel has shut down.");

            match message {
                Message::Job(job) => job(),
                Message::Terminate => break,
            }
        }
    }
}

// TODO: implement stealing work algorithm
#[inline]
pub(crate) fn spawn_worker(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
    Worker::new(id, receiver)
}
