use super::Result;
use super::ThreadPool;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;
enum Message {
    NewJob(Job),
    Terminate,
}

// storing an queue vector of workers does not work well if we want to check when a thread has panicked and we
// want to recreate a new one. The reason of this is because SharedQueueThreadPool does not allow to workers to be
// destroyed and, therefore, called the Drop trait.
// With this alternative solution a new thread can be created each time a thread has panicked. Although, I am not sure what would
// happend at the end of the server bin.
// Based on this project https://github.com/rust-threadpool/rust-threadpool/blob/master/src/lib.rs, I tried to create alternative
// channels that allows me to send messages from workers to the SharedQueueThreadPool so that new threads can be created. Nevertheless,
// it did not work.
// Also I tried to create an kind of sentinel, but the fact to share the queue vec of workers was to much difficult.

pub struct SharedQueueThreadPool {
    sender: mpsc::Sender<Message>,
}

struct Worker(Arc<Mutex<mpsc::Receiver<Message>>>);

impl Drop for Worker {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("dropped while unwinding");
            let a = Worker(self.0.clone());
            run_tasks(a);
        } else {
            println!("dropped while not unwinding");
        }
    }
}

fn run_tasks(rx: Worker) {
    thread::spawn(move || loop {
        let message =
            rx.0.lock()
                .expect("Worker thread unable to lock job_receiver")
                .recv();
        match message {
            Ok(message) => match message {
                Message::NewJob(job) => {
                    println!("Worker got a job; executing");
                    job();
                }
                Message::Terminate => {
                    println!("Worker was told to terminate");
                    break;
                }
            },
            Err(_) => {
                println!("Thread exists becuase the thread pool is destroyed");
                break;
            }
        }
    });
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        println!("Shutting down all workers");
        for _ in 0..4 {
            self.sender.send(Message::Terminate).unwrap();
        }
    }
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(size: usize) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for _ in 0..size {
            let a = Worker(receiver.clone());
            run_tasks(a);
        }
        Ok(SharedQueueThreadPool { sender })
    }

    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}
