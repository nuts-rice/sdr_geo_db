use embedded_graphics::prelude::*;

// Display dimensions for 2.8" TFT (typical resolution)
pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 240;

// Layout constants
pub const MARGIN: i32 = 4;
pub const LINE_HEIGHT: i32 = 12;
pub const HEADER_HEIGHT: i32 = 20;
pub const SECTION_SPACING: i32 = 8;

// Screen sections (Y coordinates)
pub const HEADER_Y: i32 = MARGIN;
pub const CLUSTER_Y: i32 = HEADER_Y + HEADER_HEIGHT + SECTION_SPACING;
pub const DATABASE_Y: i32 = CLUSTER_Y + (LINE_HEIGHT * 2) + SECTION_SPACING;
pub const API_Y: i32 = DATABASE_Y + (LINE_HEIGHT * 3) + SECTION_SPACING;
pub const SIGNALS_Y: i32 = API_Y + (LINE_HEIGHT * 4) + SECTION_SPACING;
pub const SYSTEM_Y: i32 = SIGNALS_Y + (LINE_HEIGHT * 4) + SECTION_SPACING;

// Node indicator positions
pub const NODE_INDICATOR_X: i32 = MARGIN + 80;
pub const NODE_INDICATOR_RADIUS: u32 = 6;
pub const NODE_INDICATOR_SPACING: i32 = 20;

/// Helper to create a point from coordinates
pub fn pt(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

/// Helper to create a size from dimensions
pub fn sz(w: u32, h: u32) -> Size {
    Size::new(w, h)
}
