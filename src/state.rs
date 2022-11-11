use cosmwasm_schema::{cw_serde};
use cw_storage_plus::{Item};


#[cw_serde]
pub struct Buffer(Vec<String>);

impl Buffer {
    pub fn new() -> Self {
        Buffer(Vec::new())
    }

    pub fn push(&mut self, value: String) {
        self.0.push(value);
    }

    pub fn pop(&mut self) -> Option<String> {
        self.0.pop()
    }
}

pub const BUFFER: Item<Buffer> = Item::new("buffer");
