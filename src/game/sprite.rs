use super::ressource_descriptor::SpriteSheetName;

pub struct SpritePosition {
    pub x: u8,
    pub y: u8,
}
impl SpritePosition {
    pub const fn new(x: u8, y: u8) -> Self {
        SpritePosition { x, y }
    }
}
pub struct SpriteDescriptor {
    sprite_sheet: SpriteSheetName,
    pub position: SpritePosition,
}
impl SpriteDescriptor {
    pub fn new(sprite_sheet: SpriteSheetName, position: SpritePosition) -> Self {
        Self { sprite_sheet, position }
    }
    pub fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.sprite_sheet
    }
}
