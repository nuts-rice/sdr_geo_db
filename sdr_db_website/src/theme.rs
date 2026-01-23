use ratatui::style::Color;
const FONT: &str = "Hack Nerd Font";

const AMBER_BRIGHT: Color = Color::Rgb(255, 170, 0); // #FFAA00
const AMBER_MID: Color = Color::Rgb(170, 102, 0); // #AA6600
const MOAB_RED: Color = Color::Rgb(77, 37, 26); // #c55f42
const MOAB_PRIMARY_TABLE_ENTRY: Color = Color::Rgb(80, 42, 20); // #cd6c35
const MOAB_SECONDARY_TABLE_ENTRY: Color = Color::Rgb(102, 51, 25); // #d9784a
const MOAB_BACKGROUND: Color = Color::Rgb(54, 28, 22); // #361c16

const CATPPUCIN_YELLOW: Color = Color::Rgb(238, 212, 159); // #EED49F
const CATPPUCIN_ORANGE: Color = Color::Rgb(245, 153, 160); // #f5a97f
const CATPPUCIN_BLUE: Color = Color::Rgb(138, 173, 244); // #8aadf4
const CATPPUCIN_PRIMARY_TABLE_ENTRY: Color = Color::Rgb(68, 71, 90); // #44475a
const CATPPUCIN_SECONDARY_TABLE_ENTRY: Color = Color::Rgb(80, 82, 100); // #505264
const CATPPUCIN_BACKGROUND: Color = Color::Rgb(40, 42, 54); // #282a36

#[derive(Debug, Clone, Default)]
pub enum Theme {
    #[default]
    Moab,
    Catppuccin,
}

impl Theme {
    pub fn as_str(&self) -> &str {
        match self {
            Theme::Moab => "Moab",
            Theme::Catppuccin => "Catppuccin",
        }
    }

    pub fn primary_color(&self) -> Color {
        match self {
            Theme::Moab => AMBER_BRIGHT,
            Theme::Catppuccin => CATPPUCIN_YELLOW,
        }
    }

    pub fn secondary_color(&self) -> Color {
        match self {
            Theme::Moab => AMBER_MID,
            Theme::Catppuccin => CATPPUCIN_ORANGE,
        }
    }

    pub fn accent_color(&self) -> Color {
        match self {
            Theme::Moab => MOAB_RED,
            Theme::Catppuccin => CATPPUCIN_BLUE,
        }
    }

    pub fn primary_table_entry_color(&self) -> Color {
        match self {
            Theme::Moab => MOAB_PRIMARY_TABLE_ENTRY,
            Theme::Catppuccin => CATPPUCIN_PRIMARY_TABLE_ENTRY,
        }
    }

    pub fn secondary_table_entry_color(&self) -> Color {
        match self {
            Theme::Moab => MOAB_SECONDARY_TABLE_ENTRY,
            Theme::Catppuccin => CATPPUCIN_SECONDARY_TABLE_ENTRY,
        }
    }

    pub fn background_color(&self) -> Color {
        match self {
            Theme::Moab => MOAB_BACKGROUND,
            Theme::Catppuccin => CATPPUCIN_BACKGROUND,
        }
    }
}
