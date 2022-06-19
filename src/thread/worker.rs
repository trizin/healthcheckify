use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
type Job = Box<dyn FnOnce() + Send + 'static>;
pub enum Message {
    NewJob(Job),
    Terminate,
}
pub struct Worker {
    pub id: u32,
    pub(crate) thread: Option<thread::JoinHandle<()>>,
    //  receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
}

impl Worker {
    pub fn new(id: u32, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        Self {
            id,
            thread: Some(thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv().unwrap();

                match job {
                    Message::NewJob(job) => {
                        job();
                    }
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);
                        break;
                    }
                }
            })),
            //receiver,
        }
    }
}
