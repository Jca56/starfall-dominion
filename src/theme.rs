//! Black with gold for what is ours, blue for the sky and the ships that sail it.
use lntrn_math::Color;
use lntrn_props::Gradient;
use lntrn_ui::Theme;

pub(crate) const GOLD: Color = Color::hex(0xE2B85C);
pub(crate) const BLUE: Color = Color::hex(0x62BBFF);
pub(crate) const INK: Color = Color::hex(0x050506);
/// Warm dark lines around panels and along the Command Strip.
pub(crate) const EDGE: Color = Color::hex(0x2E2A22);
pub(crate) const PANEL_FILL: Color = Color::hex(0x0B0B0E);
pub(crate) const TEXT: Color = Color::hex(0xF1E9D6);
pub(crate) const TEXT_DIM: Color = Color::hex(0xB3A88E);
/// Farlight's worlds.
pub(crate) const PLAYER: Color = GOLD;
/// Farlight's ships, their range, and their orders.
pub(crate) const SHIP: Color = Color::hex(0xA2E7FF);
pub(crate) const SHIP_HULL: Color = Color::hex(0x4D91BC);
/// The Starfall Dominion.
pub(crate) const DOMINION: Color = Color::hex(0xD9534F);
pub(crate) const DOMINION_HULL: Color = Color::hex(0x7A2E2B);
/// Worlds nobody holds.
pub(crate) const FREE: Color = Color::hex(0x8FA3B8);
/// A world charted from afar: position known, nothing else.
pub(crate) const CHARTED: Color = Color::hex(0x6E7684);
pub(crate) const SECTOR_LINE: Color = Color::hex(0x6A5F48);
pub(crate) const ALLOYS: Color = GOLD;
pub(crate) const ELECTRONICS: Color = Color::hex(0x7CC4E8);
pub(crate) const SECURE: Color = Color::hex(0xD7CBB0);
pub(crate) const DECAY: Color = Color::hex(0xE08A6A);
pub(crate) const WRECK: Color = Color::hex(0xC9BFAE);
pub(crate) const WARNING: Color = Color::hex(0xF3BB8F);

/// Near-black panels and controls, warm text, a gold accent, everything sized up.
pub(crate) fn theme() -> Theme {
    Theme {
        bg: INK,
        title: Gradient::new(Color::hex(0x141317), Color::hex(0x0C0C0E)),
        header: Gradient::new(Color::hex(0x17161A), Color::hex(0x0F0F12)),
        panel: Gradient::new(Color::hex(0x0E0E11), Color::hex(0x09090B)),
        widget: Gradient::new(Color::hex(0x2A282C), Color::hex(0x1B1A1E)),
        field: Color::hex(0x0A0A0C),
        text: TEXT,
        text_dim: TEXT_DIM,
        accent: GOLD,
        accent_text: Color::hex(0x1A1408),
        selection: Color::hex(0x8A6A2A),
        selection_text: TEXT,
        focus: Color::hex(0xF0C878),
        border_dark: Color::hex(0x030304),
        border_light: Color::hex(0x3E3A33),
        text_size: 30.0,
        heading_size: 45.0,
        widget_height: 65.0,
        padding: 15.0,
        gap: 10.0,
        ..Theme::default()
    }
}
