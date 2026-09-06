//! Small glyphs drawn from primitives, each with a name for its tooltip.
use lntrn_math::{Color, Rect, Vec2};
use lntrn_ui::{Sense, Ui};

use crate::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Icon {
    Alloys,
    Electronics,
    Secure,
    Decay,
}

impl Icon {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Alloys => "Alloys",
            Self::Electronics => "Advanced Electronics",
            Self::Secure => "Turns to secure",
            Self::Decay => "Influence lost per turn without a ship",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Alloys => theme::ALLOYS,
            Self::Electronics => theme::ELECTRONICS,
            Self::Secure => theme::SECURE,
            Self::Decay => theme::DECAY,
        }
    }

    /// Draw the glyph centred at `center`, about `size` physical pixels across.
    pub(crate) fn draw(self, ui: &mut Ui, center: Vec2, size: f64) {
        let line = (size * 0.09).max(1.5);
        let r = size * 0.5;
        let color = self.color();
        let at = |x: f64, y: f64| center + Vec2::new(x, y) * r;
        match self {
            Self::Alloys => {
                let corners: Vec<Vec2> = (0..6)
                    .map(|i| {
                        center
                            + Vec2::from_angle(f64::from(i) * std::f64::consts::FRAC_PI_3)
                                * r
                                * 0.95
                    })
                    .collect();
                ui.draw.polyline(&corners, line, color, true);
            }
            Self::Electronics => {
                let chip = Rect::from_center_size(center, Vec2::splat(r * 1.6));
                ui.draw.stroke_rect(chip, line, r * 0.2, color);
                ui.draw.circle(center, r * 0.25, color);
            }
            Self::Secure => {
                // A shield.
                let outline = [
                    at(-0.8, -0.75),
                    at(0.8, -0.75),
                    at(0.8, 0.05),
                    at(0.0, 0.9),
                    at(-0.8, 0.05),
                ];
                ui.draw.polyline(&outline, line, color, true);
                ui.draw.circle(at(0.0, -0.05), r * 0.18, color);
            }
            Self::Decay => {
                // An arrow draining downward.
                ui.draw.line(at(0.0, -0.85), at(0.0, 0.6), line, color);
                ui.draw.polyline(
                    &[at(-0.6, 0.05), at(0.0, 0.85), at(0.6, 0.05)],
                    line,
                    color,
                    false,
                );
            }
        }
    }

    /// Icon and text side by side in the current row, named by a tooltip.
    pub(crate) fn badge(self, ui: &mut Ui, text: &str) {
        let scale = ui.m.scale;
        let style = ui.text_style();
        let size = 30.0 * scale;
        let gap = 8.0 * scale;
        let width = size + gap + ui.measure(text, &style) + 14.0 * scale;
        let rect = ui.alloc(Vec2::new(width, ui.m.widget_h));
        let id = ui.id(self.name());
        let response = ui.interact(id, rect, Sense::NONE);
        self.draw(
            ui,
            Vec2::new(rect.min.x + size * 0.5, rect.center().y),
            size,
        );
        let label = Rect::new(Vec2::new(rect.min.x + size + gap, rect.min.y), rect.max);
        ui.text_in_rect(text, &style, label, ui.theme.text);
        ui.tooltip(&response, self.name());
    }
}
