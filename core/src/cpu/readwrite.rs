use super::*;

/// Trait implemented by objects whose registers can be accessed from the address bus
pub(crate) trait MemoryAccess {
    /// Returns value from given memory address
    fn mem_read(&self, address: u16) -> u8;
    /// Writes given value to given memory address
    fn mem_write(&mut self, address: u16, value: u8);
}

impl CPU {
    /// Reads from given memory address
    pub(crate) fn read(&self, address: u16) -> u8 {
        match address {
            // ROM, external, work and echo RAM, high RAM
            0x0000..=0x7FFF | 0xA000..=0xFDFF | 0xFF80..=0xFFFE => self.mem.mem_read(address),
            // VRAM, OAM, LCD I/O
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF4B => self.ppu.mem_read(address),
            // Audio I/O registers
            0xFF10..=0xFF3F => self.apu.mem_read(address),
            // Input register
            0xFF00 => self.input.mem_read(address),
            // Timer control
            0xFF04..=0xFF07 => self.timer.mem_read(address),
            // Interrupt control (IF and IE)
            0xFF0F | 0xFFFF => self.istate.mem_read(address),
            _ => 0xFF,
        }
    }

    /// Reads 16-bit value from given memory address
    pub(crate) fn read_16(&self, address: u16) -> u16 {
        u16::from_le_bytes([self.read(address), self.read(address + 1)])
    }

    /// Writes to given memory address
    pub(crate) fn write(&mut self, address: u16, value: u8) {
        match address {
            // ROM, external, work and echo RAM, high RAM
            0x0000..=0x7FFF | 0xA000..=0xFDFF | 0xFF80..=0xFFFE => {
                self.mem.mem_write(address, value)
            }
            // VRAM, OAM, LCD I/O
            0x8000..=0x9FFF | 0xFE00..=0xFE9F | 0xFF40..=0xFF4B => {
                self.ppu.mem_write(address, value)
            }
            // Audio I/O registers
            0xFF10..=0xFF3F => self.apu.mem_write(address, value),
            // Input register
            0xFF00 => self.input.mem_write(address, value),
            // Timer control
            0xFF04..=0xFF07 => self.timer.mem_write(address, value),
            // Interrupt control
            0xFF0F | 0xFFFF => self.istate.mem_write(address, value),
            _ => {}
        }
    }

    /// Returns the immediate 8-bit operand from memory.
    /// Increments program counter and cycles the system for one M-cycle
    pub(crate) fn read_operand(&mut self) -> u8 {
        self.cycle(1);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        self.read(self.reg.pc)
    }

    /// Returns the immediate 16-bit operand from memory.
    /// Increments program counter and cycles the system for two M-cycles
    pub(crate) fn read_operand_16(&mut self) -> u16 {
        self.cycle(2);
        self.reg.pc = self.reg.pc.wrapping_add(2);
        self.read_16(self.reg.pc - 1)
    }

    /// Pops word from memory stack and increments stack pointer.
    /// Also cycles system for two M-cycles
    pub(crate) fn pop(&mut self) -> u16 {
        self.cycle(2);
        let val = self.read_16(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(2);
        val
    }

    /// Pushes word into memory stack and decrements stack pointer
    /// Also cycles system for two M-cycles
    pub(crate) fn push(&mut self, value: u16) {
        self.cycle(2);
        let bytes = value.to_le_bytes();
        self.reg.sp = self.reg.sp.wrapping_sub(2);
        self.write(self.reg.sp.wrapping_add(1), bytes[1]);
        self.write(self.reg.sp, bytes[0]);
    }

    /// Performs an OAM DMA transfer, which copies memory from given source address to OAM
    pub(crate) fn oam_dma(&mut self, address: u8) {
        let source_address = (address as u16) * 0x100;
        for sprite_index in 0..40 {
            let sprite_address = source_address + (sprite_index * 4);
            let mut data = [0u8; 4];
            for i in 0..4u16 {
                data[i as usize] = self.read(sprite_address + i);
            }
            self.ppu.oam.sprites[sprite_index as usize] = OAMSprite::from(data);
        }
    }
}
