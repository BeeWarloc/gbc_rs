pub mod cart;
pub mod cpu;
pub mod ppu;
pub mod spu;
pub mod interconnect;
pub mod gamepad;

mod disassembler;
mod registers;
mod opcode;
mod timer;
mod mbc;

#[derive(Debug,Copy,Clone)]
pub enum GameboyType {
    Cgb,
    Dmg,
}

#[allow(dead_code)]
#[derive(Debug,Copy,Clone)]
pub enum CpuClock {
    Normal,
    Double,
}

impl CpuClock {
    #[allow(dead_code)]
    pub fn value(self) -> u32 {
        match self {
            CpuClock::Normal => 4_194_304,
            CpuClock::Double => 8_388_608,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug,Copy,Clone)]
pub enum Interrupt {
    VBlank,
    LCDStat,
    TimerOverflow,
    Serial,
    Joypad,
}

impl Interrupt {
    fn flag(self) -> u8 {
        match self {
            Interrupt::VBlank => 0b0_0001,
            Interrupt::LCDStat => 0b0_0010,
            Interrupt::TimerOverflow => 0b0_0100,
            Interrupt::Serial => 0b0_1000,
            Interrupt::Joypad => 0b1_0000,
        }
    }
}
