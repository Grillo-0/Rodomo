use std::collections::HashMap;

use crate::asc::MemoryMapped;

#[derive(Debug, Default)]
pub struct Ram {
    memory: HashMap<u16, u8>,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            memory: HashMap::new(),
        }
    }

    pub fn load_vec_at(&mut self, bytes: Vec<u8>, offset: u16) {
        for (a, v) in bytes.into_iter().enumerate() {
            self.write(a as u16 + offset, v);
        }
    }
}

impl MemoryMapped for Ram {
    fn write(&mut self, addr: u16, value: u8) {
        self.memory.insert(addr, value);
    }

    fn read(&mut self, addr: u16) -> u8 {
        let x = self.memory.get(&addr).unwrap_or(&0);
        return *x;
    }
}
