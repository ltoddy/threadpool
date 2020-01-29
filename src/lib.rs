mod pool;
mod worker;

pub use pool::ThreadPool;

enum Message {
    Job(Box<dyn FnOnce() + Send + 'static>),
    Terminate,
}
