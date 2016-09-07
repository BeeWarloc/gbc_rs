use super::interconnect::Interconnect;
use super::opcode::{OPCODE_NAME_LUT, CB_OPCODE_NAME_LUT, OPCODE_LENGTHS};
use std::string::String;

fn disassemble_opcode(opcode: u8, program_counter: u16, interconnect: &Interconnect) -> String {
    let opcode_length = OPCODE_LENGTHS[opcode as usize];
    let disasm_str = String::from(OPCODE_NAME_LUT[opcode as usize]);
    match opcode_length {
        2 => {
            let n = interconnect.read(program_counter) as u16;
            let x = format!("{:02X}", n);
            disasm_str.replace("n", &x)
        }
        3 => {
            let low = interconnect.read(program_counter) as u16;
            let high = interconnect.read(program_counter + 1) as u16;
            let x = format!("{:04X}", (high << 8) | low);
            disasm_str.replace("nn", &x)
        }
        _ => disasm_str,
    }
}

fn disassemble_cb_opcode(program_counter: u16, interconnect: &Interconnect) -> String {
    let opcode = interconnect.read(program_counter);
    String::from(CB_OPCODE_NAME_LUT[opcode as usize])
}

pub fn disassemble(program_counter: u16, interconnect: &Interconnect) -> String {

    let opcode = interconnect.read(program_counter);

    let disasm_str = {
        match opcode {
            0xcb => disassemble_cb_opcode(program_counter + 1, interconnect),
            _ => disassemble_opcode(opcode, program_counter + 1, interconnect),
        }
    };
    format!("{:04X}\t\t{}", program_counter, disasm_str)
}
