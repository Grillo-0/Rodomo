use std::collections::HashMap;
use std::fs;

#[derive(Debug, Default)]
pub struct Ram {
    memory: HashMap<u16, u8>,
}

impl Ram {
    pub fn create() -> Ram {
        Ram {
            memory: HashMap::new(),
        }
    }
    pub fn from_raw_file(path: &str) -> Ram {
        let mut ram = Ram::create();
        let bytes = fs::read(path).expect("could not read file!");
        for (a, v) in bytes.into_iter().enumerate() {
            ram.write(a.try_into().expect("Addres is higher than 16bits"), v);
        }

        return ram;
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.memory.insert(addr, value);
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let x = self.memory[&addr];
        return x;
    }
}
