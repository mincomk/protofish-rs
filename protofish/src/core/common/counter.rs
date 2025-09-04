use crate::schema::payload::schema::ContextId;

#[derive(Debug)]
pub struct ContextCounter {
    pub is_server: bool,
    counter: u64,
}

impl ContextCounter {
    pub fn new(is_server: bool) -> Self {
        Self {
            is_server,
            counter: if is_server { 1 } else { 0 },
        }
    }

    pub fn next_context_id(&mut self) -> ContextId {
        if u64::MAX - self.counter <= 2 {
            self.counter = if self.is_server { 1 } else { 2 };
            self.counter
        } else {
            self.counter += 2;
            self.counter - 2 // handshake compatible
        }
    }
}
