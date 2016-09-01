use std::string::String;
use std::boxed::Box;

#[derive(Debug)]
pub struct Cart {
    pub bytes: Box<[u8]>,
}

#[derive(Debug)]
pub enum CartType {
    RomMbc5RamBatt,
    Unsupported,
}

impl Cart {
    pub fn name(&self) -> String {
        let mut name = Vec::new();
        let mut offset = 0x0134;

        while offset <= 0x0142 {
            let byte = self.bytes[offset];

            if byte == 0x00 {
                break;
            }

            name.push(byte);
            offset = offset + 1;
        }

        String::from_utf8(name).unwrap()
    }

    pub fn cart_type(&self) -> CartType {
        match self.bytes[0x0147] {
            0x1b => CartType::RomMbc5RamBatt,
            _ => CartType::Unsupported,
        }
    }

    pub fn rom_size(&self) -> u32 {
    	match self.bytes[0x0148] {
    	    0x05 => 1024 * 1024,
    	    _ => panic!("Unsupported rom size"),
    	}
    }

    pub fn bank_count(&self) -> u32 {
    	self.rom_size() / (1024 * 16)
    }

}
