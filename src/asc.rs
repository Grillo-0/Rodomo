use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait MemoryMapped {
    fn write(&mut self, addr: u16, value: u8);

    fn read(&mut self, addr: u16) -> u8;
}

pub struct Asc {
    devices: HashMap<u16, Rc<RefCell<dyn MemoryMapped>>>,
}

impl Asc {
    pub fn new() -> Asc {
        Asc {
            devices: HashMap::new(),
        }
    }

    pub fn register_device(&mut self, addr: u16, dev: Rc<RefCell<dyn MemoryMapped>>) {
        self.devices.insert(addr, dev);
    }

    pub fn register_device_range(
        &mut self,
        addrs: impl Iterator<Item = u16>,
        dev: Rc<RefCell<dyn MemoryMapped>>,
    ) {
        for addr in addrs {
            self.devices.insert(addr, dev.clone());
        }
    }
}

impl MemoryMapped for Asc {
    fn write(&mut self, addr: u16, value: u8) {
        self.devices
            .get_mut(&addr)
            .unwrap_or_else(|| {
                panic!(
                    "tried to write value {:#x} to address {:#x} that no device is registred",
                    value, addr
                )
            })
            .borrow_mut()
            .write(addr, value);
    }

    fn read(&mut self, addr: u16) -> u8 {
        self.devices
            .get_mut(&addr)
            .unwrap_or_else(|| {
                panic!(
                    "tried to read from address {:#x} that no device is registred",
                    addr
                )
            })
            .borrow_mut()
            .read(addr)
    }
}
