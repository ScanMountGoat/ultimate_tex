use eframe::{
    egui::style::{WidgetVisuals, Widgets},
    epaint::{Color32, Rounding, Stroke},
};

const TEXT_COLOR_DARK: Color32 = Color32::from_gray(200);

pub fn widgets_dark() -> Widgets {
    // Modified from the default theme to have higher text contrast.
    Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: Color32::from_gray(27), // window background
            bg_stroke: Stroke::new(1.0, Color32::from_gray(60)), // separators, indentation lines, windows outlines
            fg_stroke: Stroke::new(1.0, TEXT_COLOR_DARK),        // normal text color
            rounding: Rounding::same(2.0),
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            bg_fill: Color32::from_gray(60), // button background
            bg_stroke: Default::default(),
            fg_stroke: Stroke::new(1.0, TEXT_COLOR_DARK), // button text
            rounding: Rounding::same(2.0),
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            bg_fill: Color32::from_gray(70),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(150)), // e.g. hover over window edge or button
            fg_stroke: Stroke::new(1.5, Color32::from_gray(255)),
            rounding: Rounding::same(3.0),
            expansion: 1.0,
        },
        active: WidgetVisuals {
            bg_fill: Color32::from_gray(55),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(255)),
            fg_stroke: Stroke::new(2.0, Color32::from_gray(255)),
            rounding: Rounding::same(2.0),
            expansion: 1.0,
        },
        open: WidgetVisuals {
            bg_fill: Color32::from_gray(27),
            bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
            fg_stroke: Stroke::new(1.0, TEXT_COLOR_DARK),
            rounding: Rounding::same(2.0),
            expansion: 0.0,
        },
    }
}
