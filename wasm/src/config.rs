use super::*;

#[derive(Debug, serde::Deserialize)]
struct SerializedRect(i16, i16, i16, i16);

impl SerializedRect {
    pub fn to_rect(&self) -> Rect {
        Rect::new(
            Vector::new(Fp::from(self.0), Fp::from(self.1)),
            Vector::new(Fp::from(self.2), Fp::from(self.3)),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
struct BackgroundDefinitionSerialized {
    controls: SerializedRect,
    display: SerializedRect,
    a: SerializedRect,
    b: SerializedRect,
    left: SerializedRect,
    right: SerializedRect,
    up: SerializedRect,
    down: SerializedRect,
    select: SerializedRect,
    start: SerializedRect,
}

/// Defines areas in the background image needed for scaling and input
#[derive(Debug, Clone)]
pub struct BackgroundDefinition {
    /// Contains the display and controls
    /// needs to be fully visible for onscreen input
    pub controls: Rect,
    pub display: Rect,
    pub a: Rect,
    pub b: Rect,
    pub left: Rect,
    pub right: Rect,
    pub up: Rect,
    pub down: Rect,
    pub select: Rect,
    pub start: Rect,
}

impl BackgroundDefinition {
    pub fn from_str(string: &str) -> Self {
        let serialized: BackgroundDefinitionSerialized = Figment::from(Toml::string(string))
            .extract()
            .expect("Couldn't deserialize background definition");
        Self::from(serialized)
    }

    pub fn get_input_rect(&self, input: InputFlag) -> Rect {
        match input {
            InputFlag::START => self.start,
            InputFlag::SELECT => self.select,
            InputFlag::A => self.a,
            InputFlag::B => self.b,
            InputFlag::UP => self.up,
            InputFlag::DOWN => self.down,
            InputFlag::LEFT => self.left,
            InputFlag::RIGHT => self.right,
            _ => unreachable!(),
        }
    }
}

impl From<BackgroundDefinitionSerialized> for BackgroundDefinition {
    fn from(value: BackgroundDefinitionSerialized) -> Self {
        Self {
            controls: value.controls.to_rect(),
            display: value.display.to_rect(),
            a: value.a.to_rect(),
            b: value.b.to_rect(),
            left: value.left.to_rect(),
            right: value.right.to_rect(),
            up: value.up.to_rect(),
            down: value.down.to_rect(),
            select: value.select.to_rect(),
            start: value.start.to_rect(),
        }
    }
}
