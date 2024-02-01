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
        let dev = self.devices.get_mut(&addr);

        if let Some(dev) = dev {
            dev.borrow_mut().write(addr, value);
        } else {
            eprintln!(
                "[WARN]: tried to write value {:#x} to address {:#x} that no device is registred",
                value, addr
            );
        }
    }

    fn read(&mut self, addr: u16) -> u8 {
        let dev = self.devices.get_mut(&addr);

        if let Some(dev) = dev {
            let value = dev.borrow_mut().read(addr);
            value
        } else {
            eprintln!(
                "[WARN]: Tried to read from address {:#x} that no device is registred",
                addr
            );
            0
        }
    }
}
