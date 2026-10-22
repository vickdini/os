use super::Message;
use spin::Mutex;

const QUEUE_CAPACITY: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointState {
    Idle,
    Active,
}

pub struct Endpoint {
    id: u32,
    buffer: [Option<Message>; QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
}

impl Endpoint {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            buffer: [None; QUEUE_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn send(&mut self, msg: Message) -> Result<(), &'static str> {
        if self.count >= QUEUE_CAPACITY {
            return Err("Endpoint message queue is full");
        }
        self.buffer[self.tail] = Some(msg);
        self.tail = (self.tail + 1) % QUEUE_CAPACITY;
        self.count += 1;
        Ok(())
    }

    pub fn receive(&mut self) -> Option<Message> {
        if self.count == 0 {
            return None;
        }
        let msg = self.buffer[self.head].take();
        self.head = (self.head + 1) % QUEUE_CAPACITY;
        self.count -= 1;
        msg
    }

    pub fn pending_count(&self) -> usize {
        self.count
    }
}

pub struct SharedEndpoint(Mutex<Endpoint>);

impl SharedEndpoint {
    pub const fn new(id: u32) -> Self {
        Self(Mutex::new(Endpoint::new(id)))
    }

    pub fn send(&self, msg: Message) -> Result<(), &'static str> {
        self.0.lock().send(msg)
    }

    pub fn receive(&self) -> Option<Message> {
        self.0.lock().receive()
    }

    pub fn id(&self) -> u32 {
        self.0.lock().id()
    }
}
