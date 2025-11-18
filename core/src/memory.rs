use super::*;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum MBCType {
    NoMBC,
    MBC1,
    MBC2,
    MMM01,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
    HuC3,
    HuC1,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CartridgeInfo {
    /// Type of memory bank controller
    pub mbc: MBCType,
    /// If cartridge provides external RAM
    pub has_ram: bool,
    /// If cartridge has battery, meaning it can store external RAM in itself
    /// (a.k.a. saving is possible)
    pub has_battery: bool,
    /// Amount of 16 KiB ROM banks cartridge provides
    pub rom_banks: u16,
    /// Amount of 8 KiB RAM banks cartridge provides
    pub ram_banks: u16,
    /// Title of the cartridge
    pub title: String,
}

impl CartridgeInfo {
    /// Returns info about cartridge features from the ROM header
    pub fn from_header(header: &[u8]) -> Self {
        let mbc = match header[0x47] {
            0x01..=0x03 => MBCType::MBC1,
            0x05..=0x06 => MBCType::MBC2,
            0x0B..=0x0D => MBCType::MMM01,
            0x0F..=0x13 => MBCType::MBC3,
            0x19..=0x1E => MBCType::MBC5,
            0x20 => MBCType::MBC6,
            0x22 => MBCType::MBC7,
            0xFE => MBCType::HuC3,
            0xFF => MBCType::HuC1,
            _ => MBCType::NoMBC,
        };
        let has_ram = matches!(
            header[0x47],
            0x02 | 0x03 | 0x0C | 0x0D | 0x10 | 0x12 | 0x13 | 0x1A | 0x1B | 0x1D | 0x1E | 0x22
        );
        let has_battery = matches!(
            header[0x47],
            0x03 | 0x06 | 0x0D | 0x0F | 0x10 | 0x13 | 0x1B | 0x1E | 0x22
        );
        let rom_banks = 2u16.saturating_pow(1 + (header[0x48] as u32));
        let ram_banks = if !has_ram {
            0
        } else {
            match header[0x49] {
                0x02 => 1,
                0x03 => 4,
                0x04 => 16,
                0x05 => 8,
                _ => 0,
            }
        };
        let title = std::str::from_utf8(&header[0x34..=0x42])
            .unwrap_or_default()
            .to_string();
        Self {
            mbc,
            has_ram,
            has_battery,
            rom_banks,
            ram_banks,
            title,
        }
    }
}

#[derive(Debug)]
pub enum MemoryInitializationErrorType {
    NoHeader,
    UnimplementedMBC(MBCType),
}

#[derive(Debug)]
pub struct MemoryInitializationError {
    error_type: MemoryInitializationErrorType,
}

impl std::fmt::Display for MemoryInitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            MemoryInitializationErrorType::NoHeader => {
                write!(f, "ROM doesn't contain header")
            }
            MemoryInitializationErrorType::UnimplementedMBC(mbc) => {
                write!(f, "MBC type {:?} isn't yet implemented. Sorry!", mbc)
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Memory {
    #[serde(with = "BigArray")]
    pub wram: [u8; 0x2000],
    #[serde(with = "BigArray")]
    pub hram: [u8; 0x7F],
    pub info: CartridgeInfo,
    mbc: MBC,
}

impl Memory {
    pub fn new(rom: Vec<u8>) -> Result<Self, MemoryInitializationError> {
        if rom.len() < 0x014F {
            return Err(MemoryInitializationError {
                error_type: MemoryInitializationErrorType::NoHeader,
            });
        }
        let info = CartridgeInfo::from_header(&rom[0x0100..=0x014F]);
        if !matches!(
            info.mbc,
            MBCType::NoMBC | MBCType::MBC1 | MBCType::MBC3 | MBCType::MBC5
        ) {
            return Err(MemoryInitializationError {
                error_type: MemoryInitializationErrorType::UnimplementedMBC(info.mbc),
            });
        }
        let mbc = MBC::init(rom, info.clone());

        Ok(Self {
            wram: [0; 0x2000],
            hram: [0; 0x7F],
            mbc,
            info,
        })
    }

    /// Overwrites ROM of simulated cartridge
    pub fn set_rom(&mut self, rom: Vec<u8>) {
        self.mbc.rom = rom;
    }

    /// Overwrites RAM of simulated cartridge
    pub fn set_ram(&mut self, ram: Vec<u8>) {
        self.mbc.ram = ram;
    }

    /// Returns copy of RAM buffer in simulated cartridge
    pub fn get_ram(&self) -> Vec<u8> {
        self.mbc.ram.clone()
    }
}

impl MemoryAccess for Memory {
    fn mem_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.mbc.read(address),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            _ => 0,
        }
    }
    fn mem_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.mbc.write(address, value),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            _ => {}
        }
    }
}

/// Simulates behavior of MBC cartridges
#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Serialize)]
struct MBC {
    // ROM is loaded manually using load_rom function
    // This is so ROM isnt also saved in the save state for no reason
    #[serde(skip_serializing, skip_deserializing)]
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    info: CartridgeInfo,
    /// Used only by MBC1
    advanced_banking: bool,
}

impl MBC {
    pub fn init(rom: Vec<u8>, info: CartridgeInfo) -> Self {
        Self {
            rom,
            ram: vec![0; usize::from(0x2000 * info.ram_banks)],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            advanced_banking: false,
            info,
        }
    }

    /// Returns value from memory at address
    /// Should handle addresses between $0000-$7FFF and $A000-$BFFF
    pub fn read(&self, address: u16) -> u8 {
        match self.info.mbc {
            MBCType::NoMBC => self.read_nombc(address),
            MBCType::MBC1 => self.read_mbc1(address),
            MBCType::MBC3 => self.read_mbc3(address),
            MBCType::MBC5 => self.read_mbc5(address),
            _ => todo!("MBC type {:?} not supported", self.info.mbc),
        }
    }
    /// Writes value into memory or register
    /// Should handle addresses between $0000-$7FFF and $A000-$BFFF
    pub fn write(&mut self, address: u16, value: u8) {
        match self.info.mbc {
            MBCType::NoMBC => self.write_nombc(address, value),
            MBCType::MBC1 => self.write_mbc1(address, value),
            MBCType::MBC3 => self.write_mbc3(address, value),
            MBCType::MBC5 => self.write_mbc5(address, value),
            _ => todo!("MBC type {:?} not supported", self.info.mbc),
        }
    }

    fn read_rom(&self, address: usize) -> u8 {
        if self.rom.len() <= address {
            log::error!(
                "Tried to read ROM at {:#06X}, but vec length is only {:#06X}",
                address,
                self.rom.len()
            );
            return 0;
        }
        self.rom[address]
    }

    fn read_ram(&self, address: usize) -> u8 {
        if self.ram.len() <= address {
            log::error!(
                "Tried to read RAM at {:#06X}, but vec length is only {:#06X}",
                address,
                self.ram.len()
            );
            return 0;
        }
        self.ram[address]
    }

    fn write_ram(&mut self, address: usize, value: u8) {
        if self.ram.len() <= address {
            log::error!(
                "Tried to write to RAM at {:#06X}, but vec length is only {:#06X}",
                address,
                self.ram.len()
            );
            return;
        }
        self.ram[address] = value;
    }

    /// Used to mask bank number register value to wrap around
    /// based on maximum number of banks
    fn mask_bank_number(&self, number: u8, bank_amount: u16) -> usize {
        // Calculate amount of bits needed to contain number
        // Clamp result to 8, when bit amount is higher the higher bits are written to another
        // register
        let bit_amount = (bank_amount as f32).log(2.0).ceil().min(8.0) as usize;
        // Mask number with amount of bits
        if bit_amount == 0 {
            0
        } else {
            (number as usize) & (0xFF >> (8 - bit_amount))
        }
    }

    fn read_nombc(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.read_rom(address as usize),
            0xA000..=0xBFFF => self.read_ram(address as usize),
            _ => 0xFF,
        }
    }

    fn write_nombc(&mut self, address: u16, value: u8) {
        if matches!(address, 0xA000..=0xBFFF) {
            self.write_ram(address as usize, value);
        }
    }

    fn read_mbc1(&self, address: u16) -> u8 {
        let mut address = address as usize;
        match address {
            0x0000..=0x7FFF => {
                // The ROM bank register is only applied
                // to the second ROM address range ($4000-$7FFF)
                if address >= 0x4000 {
                    address -= 0x4000;
                    address += self.rom_bank * 0x4000;
                }
                // If cartridge has >512 KiB ROM, the 2-bit register that is also used to select RAM banks
                // can be used to select one of four large banks of 512 KiB memory
                // It is also applied to the first address range if using advanced banking mode
                if self.info.rom_banks > 32 && (address >= 0x4000 || self.advanced_banking) {
                    // Mask out upper bit of high address if not enough banks
                    let high_address = self.ram_bank
                        & if self.info.rom_banks <= 64 {
                            0b01
                        } else {
                            0b11
                        };
                    address += 0x20 * high_address * 0x4000
                }
                self.read_rom(address)
            }
            0xA000..=0xBFFF => {
                // Reads to disabled RAM usually return 0xFF
                if !self.ram_enabled {
                    return 0xFF;
                }
                address -= 0xA000;
                // RAM banks can only be changed when using advanced banking mode
                if self.advanced_banking {
                    address +=
                        self.mask_bank_number(self.ram_bank as u8, self.info.ram_banks) * 0x2000;
                }
                self.read_ram(address)
            }
            _ => 0xFF,
        }
    }

    fn write_mbc1(&mut self, address: u16, value: u8) {
        match address {
            // Enable the RAM
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            // Less significant ROM bank register
            0x2000..=0x3FFF => {
                // Only needed amount of bits to change between all ROM banks
                // are saved to the register, rest are masked out
                let mut masked = self.mask_bank_number(value, self.info.rom_banks.clamp(0, 32));
                // If register is tried to set to 0, it should be incremented to 1
                // The check is only done for the 5-bit version for the value though,
                // so for example if only 3 bits are used,
                // register value 0b1000 maps the second block to ROM bank 0
                if value & 0b1_1111 == 0 {
                    masked += 1;
                }
                self.rom_bank = masked;
            }
            // 2 bit bank register that is used to select both ROM and RAM banks
            0x4000..=0x5FFF => self.ram_bank = (value & 0b11) as usize,
            // Toggle between banking modes
            // The above register only has effect if this is set to true
            0x6000..=0x7FFF => self.advanced_banking = value & 0b1 > 0,
            // Write to RAM
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }
                let mut address = address as usize;
                address -= 0xA000;
                // RAM banks can only be changed when using advanced banking mode
                if self.advanced_banking && self.info.ram_banks > 1 {
                    address += self.ram_bank * 0x2000;
                }
                self.write_ram(address, value);
            }
            _ => {}
        };
    }

    fn read_mbc3(&self, address: u16) -> u8 {
        let mut address = address as usize;
        match address {
            0x0000..=0x3FFF => self.rom[address],
            0x4000..=0x7FFF => {
                address += 0x4000 * (self.rom_bank - 1);
                self.read_rom(address)
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }
                address -= 0xA000;
                address += self.ram_bank * 0x2000;
                self.read_ram(address)
            }
            _ => 0xFF,
        }
    }

    fn write_mbc3(&mut self, address: u16, value: u8) {
        match address {
            // RAM enabled
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            // ROM bank number
            0x2000..=0x3FFF => {
                let mut masked = self.mask_bank_number(value, self.info.rom_banks);
                if masked == 0 {
                    masked = 1
                };
                self.rom_bank = masked;
            }
            // RAM bank number
            0x4000..=0x5FFF => {
                if self.info.ram_banks != 0 {
                    self.ram_bank = self.mask_bank_number(value, self.info.ram_banks);
                }
            }
            // Write to RAM
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }
                let mut address = address as usize;
                address -= 0xA000;
                address += self.ram_bank * 0x2000;
                self.write_ram(address, value);
            }
            _ => {}
        };
    }

    fn read_mbc5(&self, address: u16) -> u8 {
        let mut address = address as usize;
        match address {
            0x0000..=0x3FFF => self.rom[address],
            0x4000..=0x7FFF => {
                address = address.saturating_add_signed(0x4000 * ((self.rom_bank as isize) - 1));
                self.read_rom(address)
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }
                address -= 0xA000;
                address += self.ram_bank * 0x2000;
                self.read_ram(address)
            }
            _ => 0xFF,
        }
    }

    fn write_mbc5(&mut self, address: u16, value: u8) {
        match address {
            // RAM enabled
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            // 8 least significant bits of ROM bank number
            0x2000..=0x2FFF => {
                let masked = self.mask_bank_number(value, self.info.rom_banks);
                self.rom_bank = masked;
            }
            // 9th bit of ROM bank number
            0x3000..=0x3FFF => self.rom_bank |= ((value & 1) as usize) << 8,
            // RAM bank number
            0x4000..=0x5FFF => {
                if self.info.ram_banks != 0 {
                    self.ram_bank = self.mask_bank_number(value, self.info.ram_banks);
                }
            }
            // Write to RAM
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return;
                }
                let mut address = address as usize;
                address -= 0xA000;
                address += self.ram_bank * 0x2000;
                self.write_ram(address, value);
            }
            _ => {}
        };
    }
}
