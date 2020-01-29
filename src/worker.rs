use std::collections::VecDeque;
use std::sync::{
    mpsc::Receiver,
    {Arc, Mutex},
};
use std::thread;

use super::Message;

pub(crate) struct Worker {
    _id: usize,
    receiver_thread: Option<thread::JoinHandle<()>>,
    worker_threads: Vec<Option<thread::JoinHandle<()>>>,
}

impl Worker {
    pub(crate) fn new(id: usize, workers: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::with_capacity(64)));

        let receiver_thread = {
            let queue = Arc::clone(&queue);
            thread::spawn(move || Self::receive_message(receiver, queue))
        };

        let mut worker_threads = Vec::<Option<thread::JoinHandle<()>>>::with_capacity(workers);
        for _ in 0..workers {
            let queue = Arc::clone(&queue);
            let worker_thread = thread::spawn(move || Self::work(queue));
            worker_threads.push(Some(worker_thread));
        }

        Self {
            _id: id,
            receiver_thread: Some(receiver_thread),
            worker_threads,
        }
    }

    fn receive_message(
        receiver: Arc<Mutex<Receiver<Message>>>,
        queue: Arc<Mutex<VecDeque<Message>>>,
    ) {
        let message = receiver
            .lock()
            .expect("Can't lock thread")
            .recv()
            .expect("channel had shutdown");

        queue.lock().expect("Can't lock queue").push_back(message);
    }

    fn work(queue: Arc<Mutex<VecDeque<Message>>>) {
        loop {
            if let Some(message) = queue.lock().expect("Can't lock queue").pop_front() {
                match message {
                    Message::Job(job) => job(),
                    Message::Terminate => break,
                }
            }
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(thread) = self.receiver_thread.take() {
            thread.join().unwrap();
        }

        for worker_thread in &mut self.worker_threads {
            if let Some(worker_thread) = worker_thread.take() {
                worker_thread.join().unwrap();
            }
        }
    }
}
