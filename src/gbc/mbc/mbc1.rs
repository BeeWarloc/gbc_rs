use super::Mbc;
use super::MbcInfo;

#[derive(Debug)]
pub struct Mbc1 {
    ram_write_protected: bool,
    rom_bank_0: u8,
    rom_bank_1: u8,
    ram_select: u8,
    rom_offset: usize,
    ram_offset: usize,
    ram: Box<[u8]>,
}

impl Mbc1 {
    pub fn new(mbc_info: MbcInfo) -> Mbc1 {
        Mbc1 {
            ram_write_protected: true,
            rom_bank_0: 0,
            rom_bank_1: 0,
            ram_select: 0,
            rom_offset: 0,
            ram_offset: 0,
            ram: if let Some(ram_info) = mbc_info.ram_info {
                vec![0; ram_info.size as usize].into_boxed_slice()
            } else {
                vec![0; 0].into_boxed_slice()
            },
        }
    }

    fn update_rom_offset(&mut self) {
        let bank_0 = if self.rom_bank_0 & 0x0f == 0 {
            self.rom_bank_0 | 0x01
        } else {
            self.rom_bank_0
        } as usize;

        let bank_1 = if self.ram_select == 0 {
            self.rom_bank_1
        } else {
            0
        } as usize;

        self.rom_offset = bank_0 * 0x4000 + bank_1 * 512 * 1024;
    }

    fn update_ram_offset(&mut self) {
        self.ram_offset = if self.ram_select == 1 {
            self.rom_bank_1 as usize * 8 * 1024
        } else {
            0
        };
    }
}

impl Mbc for Mbc1 {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8 {
        match addr {
            0x0000...0x3fff => rom[addr as usize],
            0x4000...0x7fff => rom[addr as usize - 0x4000 + self.rom_offset],
            _ => panic!("Address out of range 0x{:x}", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x1fff => self.ram_write_protected = val == 0x0a,
            0x2000...0x3fff => self.rom_bank_0 = val,
            0x4000...0x5fff => self.rom_bank_1 = val,
            0x6000...0x7fff => self.ram_select = val,
            _ => panic!("Illegal address: 0x{:x}", addr),
        }
        self.update_rom_offset();
        self.update_ram_offset()
    }

    fn read_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize - 0xa000 + self.ram_offset]
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize - 0xa000 + self.ram_offset] = val
    }
}
