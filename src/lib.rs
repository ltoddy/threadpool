use std::sync::{
    mpsc::{sync_channel, Receiver, SyncSender},
    {Arc, Mutex},
};
use std::thread;

enum Message {
    Job(Box<dyn FnOnce() + Send + 'static>),
    Terminate,
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = rx
                .lock()
                .expect("Can't lock thread")
                .recv()
                .expect("channel has shutdown");

            match message {
                Message::Job(job) => job(),
                Message::Terminate => break,
            }
        });

        Self {
            _id: id,
            thread: Some(thread),
        }
    }
}

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
            workers.push(Worker::new(i, Arc::clone(&receiver)));
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

impl Drop for ThreadPool {
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
