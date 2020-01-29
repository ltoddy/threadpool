use std::sync::{
    mpsc::{sync_channel, SyncSender},
    {Arc, Mutex},
};

use super::Message;
use crate::worker::spawn_worker;
use crate::worker::Worker;

pub struct ThreadPoolExecutor {
    workers: Vec<Worker>,
    sender: SyncSender<Message>,
}

impl ThreadPoolExecutor {
    pub fn new() -> Self {
        // TODO: capacity should calculate from the number of cpus
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0);

        // TODO: use config file (Cargo.toml) to set sync_channel buffer size
        let (sender, receiver) = sync_channel::<Message>(1024);
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::<Worker>::with_capacity(capacity);

        for i in 0..capacity {
            workers.push(spawn_worker(i, Arc::clone(&receiver)));
        }

        Self { workers, sender }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Message::Job(Box::new(f));

        self.sender.send(job).unwrap();
    }
}

impl Default for ThreadPoolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ThreadPoolExecutor {
    fn drop(&mut self) {
        for _ in 0..self.workers.len() {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
