use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_schema::{cw_serde};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};


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
