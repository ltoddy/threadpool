mod executor;
mod worker;

pub use executor::ThreadPoolExecutor;

enum Message {
    Job(Box<dyn FnOnce() + Send + 'static>),
    Terminate,
}
