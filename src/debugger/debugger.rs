use crate::gbr::game_boy::GameBoy;
use crate::gbr::instruction::Instruction;

pub struct Debugger {
    emu: GameBoy,
}

impl Debugger {
    pub fn new(game_boy: GameBoy) -> Self {
        Debugger { emu: game_boy }
    }

    pub fn step(&mut self) {
        self.emu.step();
    }

    pub fn disassemble(&self) -> Vec<(u16, Instruction)> {
        let cpu = self.emu.cpu();
        let mut pc = cpu.read_pc();

        let mut disassembly = Vec::<(u16, Instruction)>::new();

        for _ in 0..20 {
            let instr = self.emu.bus().fetch_instruction(pc);
            let new_pc = pc + instr.length();
            disassembly.push((pc, instr));
            pc = new_pc;
        }

        disassembly
    }
}
