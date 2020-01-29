use std::sync::{
    mpsc::{sync_channel, SyncSender},
    {Arc, Mutex},
};

use super::Message;
use crate::worker::Worker;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: SyncSender<Message>,
}

impl ThreadPool {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0);

        // TODO: use config file (Cargo.toml) to set sync_channel buffer size
        let (sender, receiver) = sync_channel::<Message>(1024);
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::<Worker>::with_capacity(capacity);

        for i in 0..capacity {
            // TODO: remove magic number
            workers.push(Worker::new(i, 4, Arc::clone(&receiver)));
        }

        Self { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Message::Job(Box::new(f));

        self.sender.send(job).unwrap();
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            self.sender.send(Message::Terminate).unwrap();
        }
    }
}
