use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

pub trait MemoryMapped: Debug {
    fn write(&mut self, addr: u16, value: u8);

    fn read(&mut self, addr: u16) -> u8;
}

#[derive(Debug)]
pub struct Asc {
    devices: HashMap<u16, Rc<RefCell<dyn MemoryMapped>>>,
    mirror_masks: HashMap<u16, u16>,
}

impl Asc {
    pub fn new() -> Asc {
        Asc {
            devices: HashMap::new(),
            mirror_masks: HashMap::new(),
        }
    }

    pub fn register_device(
        &mut self,
        addr: u16,
        dev: Rc<RefCell<dyn MemoryMapped>>,
        mirror_mask: u16,
    ) {
        self.devices.insert(addr, dev);
        self.mirror_masks.insert(addr, mirror_mask);
    }

    pub fn register_device_range(
        &mut self,
        addrs: impl Iterator<Item = u16>,
        dev: Rc<RefCell<dyn MemoryMapped>>,
        mirror_mask: u16,
    ) {
        for addr in addrs {
            self.devices.insert(addr, dev.clone());
            self.mirror_masks.insert(addr, mirror_mask);
        }
    }
}

impl MemoryMapped for Asc {
    fn write(&mut self, mut addr: u16, value: u8) {
        let dev = self.devices.get_mut(&addr);

        let mirror_mask = self.mirror_masks.get(&addr);

        if let Some(mask) = mirror_mask {
            addr &= mask;
        }

        if let Some(dev) = dev {
            dev.borrow_mut().write(addr, value);
        } else {
            eprintln!(
                "[WARN]: tried to write value {:#x} to address {:#x} that no device is registred",
                value, addr
            );
        }
    }

    fn read(&mut self, mut addr: u16) -> u8 {
        let dev = self.devices.get_mut(&addr);

        let mirror_mask = self.mirror_masks.get(&addr);

        if let Some(mask) = mirror_mask {
            addr &= mask;
        }

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
