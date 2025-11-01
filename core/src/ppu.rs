use super::*;
use double_buffer::DoubleBuffer;

/// LCDC register
#[derive(Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct LCDControl(u8);

/// Determines which sources to use for STAT interrupt
#[derive(Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct STATEnable(u8);

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct SpriteFlags(u8);

bitflags! {
    impl LCDControl: u8 {
        /// If PPU is enabled
        const ENABLE              = 0b1000_0000;
        /// Chooses between two tile map areas to use for drawing window
        const WINDOW_TILE_MAP     = 0b0100_0000;
        /// If window should be drawn
        const WINDOW_ENABLE       = 0b0010_0000;
        /// Chooses between two tile data areas to use for drawing
        /// window and background
        const TILE_DATA_AREA      = 0b0001_0000;
        /// Chooses between two tile map areas to use for drawing background
        const BG_TILE_MAP         = 0b0000_1000;
        /// Chooses whether to use larger sized objects (8x16) instead of regular ones (8x8)
        const OBJ_SIZE            = 0b0000_0100;
        /// If objects should be drawn
        const OBJ_ENABLE          = 0b0000_0010;
        /// Disables both background and window when 0
        const BG_WINDOW_ENABLE    = 0b0000_0001;
    }

    impl STATEnable: u8 {
        // When scanline hits value in LYC register
        const LYC   = 0b0100_0000;
        // When PPU starts drawing new line
        const Mode2 = 0b0010_0000;
        // When VBlank is hit and frame is fully drawn
        const Mode1 = 0b0001_0000;
        // When HBlank is hit and scanline is fully drawn
        const Mode0 = 0b0000_1000;
    }

    impl SpriteFlags: u8 {
        /// If true, background and window is drawn over sprite
        const PRIORITY = 0b1000_0000;
        /// If sprite should be flipped vertically
        const Y_FLIP   = 0b0100_0000;
        /// If sprite should be flipped horizontally
        const X_FLIP   = 0b0010_0000;
        /// Which palette to use
        const PALETTE  = 0b0001_0000;
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct OAMSprite {
    pub y: u8,
    pub x: u8,
    pub tile_index: u8,
    pub flags: SpriteFlags,
}

impl OAMSprite {
    pub fn from(data: [u8; 4]) -> Self {
        Self {
            y: data[0],
            x: data[1],
            tile_index: data[2],
            flags: SpriteFlags::from_bits_retain(data[3]),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Serialize)]
pub struct OAM {
    #[serde(with = "BigArray")]
    pub sprites: [OAMSprite; 40],
}

impl OAM {
    pub fn new() -> Self {
        Self {
            sprites: [OAMSprite::from([0; 4]); 40],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        let sprite = &self.sprites[usize::from(address / 4)];
        match address % 4 {
            0 => sprite.y,
            1 => sprite.x,
            2 => sprite.tile_index,
            3 => sprite.flags.bits(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let sprite = &mut self.sprites[usize::from(address / 4)];
        match address % 4 {
            0 => sprite.y = value,
            1 => sprite.x = value,
            2 => sprite.tile_index = value,
            3 => sprite.flags = SpriteFlags::from_bits_retain(value),
            _ => unreachable!(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct DMGPalettes {
    bg: u8,
    obj0: u8,
    obj1: u8,
}

/// The Game Boy's 160x144 display has 23040 pixels that can
/// display 4 colors (represented in two bits).
/// This requires 46080 bits which are represented as 1440 unsigned 32-bit integers.
pub const DISPLAY_BUFFER_SIZE: usize = (2 * 160 * 144) / 32;

/// Buffer representing the Game Boy's display.
/// Each pixel takes two bits and the integer's least significant bits represent the leftmost
/// pixels.
pub type DisplayBuffer = [u32; DISPLAY_BUFFER_SIZE];

fn empty_display() -> DoubleBuffer<DisplayBuffer> {
    DoubleBuffer::new([0; DISPLAY_BUFFER_SIZE], [0; DISPLAY_BUFFER_SIZE])
}

/// Describes the current drawing state of the PPU
#[derive(Deserialize, Serialize, PartialEq, Clone, Copy)]
pub enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAMScan = 2,
    Drawing = 3,
}

impl From<PPUMode> for u8 {
    fn from(src: PPUMode) -> u8 {
        use PPUMode::*;
        match src {
            HBlank => 0,
            VBlank => 1,
            OAMScan => 2,
            Drawing => 3,
        }
    }
}

/// Describes the active state of the PPU
#[derive(Deserialize, Serialize, PartialEq, Clone, Copy)]
pub enum PPUState {
    /// Active and drawing normally
    Active,
    /// Disabled, no registers are updated, display shows white
    Disabled,
    /// Set active this frame, registers are updated, display shows white until next frame
    Starting,
}

/// The graphics processing unit
#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Serialize)]
pub struct PPU {
    pub state: PPUState,
    #[serde(skip)]
    #[serde(default = "empty_display")]
    pub display: DoubleBuffer<DisplayBuffer>,
    #[serde(with = "BigArray")]
    pub vram: [u8; 0x2000],
    pub oam: OAM,
    /// The source address for OAM DMA tranfer
    pub oam_dma_source: u8,
    /// If true, an OAM DMA transfer is requested.
    /// This is set back to false when transfer begins on CPU
    pub oam_dma_request: bool,
    /// LCD control register
    pub lcdc: LCDControl,
    /// The current horizontal scan position
    pub lx: u16,
    /// The current scanline
    pub ly: u8,
    /// Background scroll position X
    pub bg_x: u8,
    /// Background scroll position Y
    pub bg_y: u8,
    /// Window position X
    pub win_x: u8,
    /// Window position Y
    pub win_y: u8,
    /// Window line counter
    pub win_line: u8,
    /// The current palettes used for rendering
    pub palettes: DMGPalettes,
    /// Currently requested interrupt.
    /// Is reset to 0 after handling on CPU
    pub interrupt_request: InterruptFlag,
    pub stat_enable: STATEnable,
    /// Current drawing state
    pub mode: PPUMode,
    /// Register to compare to scanline coordinate for interrupts
    pub lyc: u8,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            state: PPUState::Active,
            display: empty_display(),
            vram: [0; 0x2000],
            oam: OAM::new(),
            oam_dma_source: 0,
            oam_dma_request: false,
            lcdc: LCDControl::from_bits_truncate(0b0000_0000),
            lx: 0,
            ly: 0,
            bg_x: 0,
            bg_y: 0,
            win_x: 0,
            win_y: 0,
            win_line: 0,
            palettes: DMGPalettes {
                bg: 0,
                obj0: 0,
                obj1: 0,
            },
            interrupt_request: InterruptFlag::from_bits_truncate(0),
            stat_enable: STATEnable::from_bits_truncate(0),
            mode: PPUMode::OAMScan,
            lyc: 0,
        }
    }

    pub fn cycle(&mut self) {
        use {PPUMode::*, PPUState::*};
        self.interrupt_request = InterruptFlag::from_bits_truncate(0);

        if self.state == Disabled {
            return;
        }

        if self.lx < 455 {
            self.lx += 1;
            if self.mode != VBlank {
                if self.lx == 80 {
                    self.update_mode(Drawing);
                } else if self.lx == 80 + 172 {
                    self.update_mode(HBlank);
                }
            }
        } else {
            self.lx = 0;

            match self.ly {
                0..=143 => {
                    // Draw new line
                    self.update_mode(OAMScan);
                    self.draw_scanline(self.ly)
                }
                144 => {
                    // Reset line counter
                    self.win_line = 0;
                    // Send VBlank interrupt
                    self.interrupt_request.insert(InterruptFlag::VBLANK);
                    self.update_mode(VBlank);
                    // Swap double buffer for rendering new frame
                    self.display.swap();
                }
                153 => {
                    // Start drawing new frame
                    self.ly = 0;
                    self.update_mode(OAMScan);
                    // End drawing delay after PPU was enabled again
                    if self.state == Starting {
                        self.state = Active;
                    }
                    return;
                }
                _ => {}
            }
            self.ly += 1;

            // Check for LYC=LY interrupt if its enabled
            if self.stat_enable.intersects(STATEnable::LYC) && self.lyc == self.ly {
                self.interrupt_request.insert(InterruptFlag::LCD);
            }
        }
    }

    fn update_mode(&mut self, mode: PPUMode) {
        use PPUMode::*;
        self.mode = mode;
        // Send interrupt if enabled
        if (mode == OAMScan && self.stat_enable.intersects(STATEnable::Mode2))
            || (mode == VBlank && self.stat_enable.intersects(STATEnable::Mode1))
            || (mode == HBlank && self.stat_enable.intersects(STATEnable::Mode0))
        {
            self.interrupt_request.insert(InterruptFlag::LCD);
        }
    }

    /// Get color value from given palette
    fn get_palette_color(&mut self, col_id: u8, palette: u8) -> u8 {
        (palette >> (2 * col_id)) & 0b11
    }

    /// Saves given color into the display buffer
    fn set_pixel(&mut self, x: u8, y: u8, col: u8) {
        // When PPU has been re-enabled this frame,
        // the display stays white until next frame
        if self.state == PPUState::Starting {
            return;
        }
        let i = (((y as usize) * 2 * 160) + ((x as usize) * 2)) / 32;
        let shift = (x % 16) * 2;
        self.display[i] &= !(0b11 << shift);
        self.display[i] |= (col as u32) << shift;
    }

    /// Returns the tile index from specified tile map at specified coordinates
    fn get_tile_index(&self, x: u8, y: u8, tile_map: bool) -> u8 {
        // Get tile index in tile map
        let tile_map_index = u16::from(y / 8)
            .wrapping_mul(32)
            .wrapping_add((x / 8) as u16);
        // Get start position in memory of selected tile map
        let tile_map_root: u16 = if tile_map { 0x1C00 } else { 0x1800 };
        // Get index of tile data from tile map
        self.vram[usize::from(tile_map_root + tile_map_index)]
    }

    /// Returns the color ID of given tile at specified coordinates
    fn get_tile_color(&self, x: u8, y: u8, tile_index: u8, addressing_mode: bool) -> u8 {
        // Get memory position of tile inside VRAM (one tile is 16 bytes)
        // and add target row to it to get address of the two bytes
        // that make up a tile row of color data
        let mut byte_index = (16 * (tile_index as u16)) + (2 * ((y as u16) % 8));

        // When using alternative addressing mode, tile data at index < 128
        // are found at address 0x9000-0x97FF
        if addressing_mode && tile_index < 128 {
            byte_index += 0x1000;
        }

        // Get the color bytes from VRAM
        let a_byte = self.vram[usize::from(byte_index)];
        let b_byte = self.vram[usize::from(byte_index + 1)];
        // Get the bit values of correct tile column
        let a = a_byte & (0b1000_0000 >> (x % 8)) != 0;
        let b = b_byte & (0b1000_0000 >> (x % 8)) != 0;

        // Get color ID from the two bits
        (a as u8) | ((b as u8) << 1)
    }

    /// Returns list of sprites that occupy given scanline
    fn get_sprites(&self, y: u8, sprite_height: u8) -> Vec<OAMSprite> {
        // Convert screen Y to object space,
        // where y = 0 completely hides the object
        let obj_y = y + 16;

        let mut sprites: Vec<OAMSprite> = vec![];
        for sprite in &self.oam.sprites {
            if obj_y < sprite.y.saturating_add(sprite_height) && obj_y >= sprite.y {
                sprites.push(*sprite);
                // There's a limit of 10 objects per scanline
                if sprites.len() == 10 {
                    break;
                }
            }
        }
        // Sort sprites by their x coordinate,
        // giving render priority to the sprite with the smallest x
        sprites.sort_by_key(|sprite| sprite.x);
        sprites
    }

    fn draw_scanline(&mut self, y: u8) {
        // Get object height based on current LCD control
        let sprite_height = if self.lcdc.intersects(LCDControl::OBJ_SIZE) {
            16
        } else {
            8
        };

        let sprites = self.get_sprites(y, sprite_height);
        for x in 0..=159u8 {
            let mut drawn_sprite: Option<&OAMSprite> = None;
            let mut sprite_col = 0u8;
            if self.lcdc.intersects(LCDControl::OBJ_ENABLE) {
                // Convert screen X to object space
                let obj_x = x + 8;
                for sprite in &sprites {
                    if obj_x < sprite.x.saturating_add(8) && obj_x >= sprite.x {
                        // Calculate X coordinate inside sprite
                        let mut tile_x = (x as i16) - ((sprite.x as i16) - 8);
                        tile_x %= 8;
                        if sprite.flags.intersects(SpriteFlags::X_FLIP) {
                            tile_x = 7 - tile_x;
                        }

                        // Calculate Y coordinate inside sprite
                        let mut tile_y = (y as i16) - ((sprite.y as i16) - 16);
                        tile_y %= sprite_height as i16;
                        if sprite.flags.intersects(SpriteFlags::Y_FLIP) {
                            tile_y = (sprite_height as i16) - 1 - tile_y;
                        }

                        let mut tile_index = sprite.tile_index;
                        if self.lcdc.intersects(LCDControl::OBJ_SIZE) {
                            // 8x16 objects ignore last bit of tile index
                            tile_index &= 0b1111_1110;
                            // Read bottom pixels of 8x16 object from the tile at next index
                            if tile_y >= 8 {
                                tile_y -= 8;
                                tile_index += 1;
                            }
                        }

                        let col_id =
                            self.get_tile_color(tile_x as u8, tile_y as u8, tile_index, false);
                        // With objects, color ID of 0 means transparent,
                        // so render background or window instead
                        if col_id != 0 {
                            drawn_sprite = Some(sprite);
                            let palette = if sprite.flags.intersects(SpriteFlags::PALETTE) {
                                self.palettes.obj1
                            } else {
                                self.palettes.obj0
                            };
                            sprite_col = self.get_palette_color(col_id, palette);
                            break;
                        }
                    }
                }
            }

            let mut sprite_on_background = false;
            if let Some(sprite) = drawn_sprite {
                // If object priority flag is true,
                // background / window can be rendered on top of it
                if !sprite.flags.intersects(SpriteFlags::PRIORITY) {
                    self.set_pixel(x, y, sprite_col);
                    continue;
                }
                sprite_on_background = true;
            }

            // If background and window are disabled, render object or just blank
            if !self.lcdc.intersects(LCDControl::BG_WINDOW_ENABLE) {
                if sprite_on_background {
                    self.set_pixel(x, y, sprite_col);
                } else {
                    let col = self.get_palette_color(0, self.palettes.bg);
                    self.set_pixel(x, y, col);
                }
                continue;
            }

            // Get window pixel instead of background if
            // window is enabled and pixel is inside window bounds
            let col_id = if self.lcdc.intersects(LCDControl::WINDOW_ENABLE)
                && x >= self.win_x
                && y >= self.win_y
            {
                let tile = self.get_tile_index(
                    x - self.win_x,
                    self.win_line,
                    self.lcdc.intersects(LCDControl::WINDOW_TILE_MAP),
                );
                self.get_tile_color(
                    x - self.win_x,
                    self.win_line,
                    tile,
                    !self.lcdc.intersects(LCDControl::TILE_DATA_AREA),
                )
            } else {
                // Coordinates of background tiles may wrap around
                let tile = self.get_tile_index(
                    x.wrapping_add(self.bg_x),
                    y.wrapping_add(self.bg_y),
                    self.lcdc.intersects(LCDControl::BG_TILE_MAP),
                );
                self.get_tile_color(
                    x.wrapping_add(self.bg_x),
                    y.wrapping_add(self.bg_y),
                    tile,
                    !self.lcdc.intersects(LCDControl::TILE_DATA_AREA),
                )
            };
            // If pixel color ID is 0, render sprite instead
            if col_id == 0 && sprite_on_background {
                self.set_pixel(x, y, sprite_col);
            }
            // Otherwise render background / window pixel
            else {
                let col = self.get_palette_color(col_id, self.palettes.bg);
                self.set_pixel(x, y, col);
            }
        }
        // Increment line counter if window was displayed on this scanline
        if self.lcdc.intersects(LCDControl::WINDOW_ENABLE) && self.win_x < 160 && self.win_y <= y {
            self.win_line += 1;
        }
    }

    fn disable(&mut self) {
        if self.state == PPUState::Disabled {
            return;
        }
        // Clear the display only if it has been drawn to since last disable
        if self.state == PPUState::Active {
            self.display.fill(0);
            self.display.swap();
            self.display.fill(0);
        }
        self.state = PPUState::Disabled;
        // Clear registers
        self.mode = PPUMode::HBlank;
        self.lx = 0;
        self.ly = 0;
    }
}

impl MemoryAccess for PPU {
    fn mem_read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam.read(address - 0xFE00),
            0xFF40 => self.lcdc.bits(),
            0xFF41 => {
                let lyc = ((self.lyc == self.ly) as u8) << 2;
                self.stat_enable.bits() | lyc | u8::from(self.mode)
            }
            0xFF42 => self.bg_y,
            0xFF43 => self.bg_x,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.oam_dma_source,
            0xFF47 => self.palettes.bg,
            0xFF48 => self.palettes.obj0,
            0xFF49 => self.palettes.obj1,
            0xFF4A => self.win_y,
            0xFF4B => self.win_x + 7,
            _ => 0xFF,
        }
    }
    fn mem_write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xFE00..=0xFE9F => self.oam.write(address - 0xFE00, value),
            0xFF40 => {
                self.lcdc = LCDControl::from_bits_truncate(value);
                let enabled = self.lcdc.intersects(LCDControl::ENABLE);
                if self.state != PPUState::Disabled && !enabled {
                    self.disable();
                } else if self.state == PPUState::Disabled && enabled {
                    self.state = PPUState::Starting;
                    self.mode = PPUMode::OAMScan;
                }
            }
            0xFF41 => self.stat_enable = STATEnable::from_bits_truncate(value),
            0xFF42 => self.bg_y = value,
            0xFF43 => self.bg_x = value,
            0xFF45 => self.lyc = value,
            0xFF46 => {
                self.oam_dma_source = value;
                self.oam_dma_request = true;
            }
            0xFF47 => self.palettes.bg = value,
            0xFF48 => self.palettes.obj0 = value,
            0xFF49 => self.palettes.obj1 = value,
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value.saturating_sub(7),
            _ => {}
        }
    }
}
