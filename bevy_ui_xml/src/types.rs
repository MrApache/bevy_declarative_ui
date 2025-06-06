use bevy::prelude::*;

pub fn parse_flex_direction(str: &str) -> FlexDirection {
    match str {
        "Row"           => FlexDirection::Row,
        "Column"        => FlexDirection::Column,
        "RowReverse"    => FlexDirection::RowReverse,
        "ColumnReverse" => FlexDirection::ColumnReverse,
        _ =>  {
            error!("Unknown flex direction value: {}", str);
            FlexDirection::default()
        }
    }
}

pub fn color_str(value: &str) -> Color {
    let value: &str = value.trim();

    if let Some(hex) = value.strip_prefix('#') {
        return match hex.len() {
            6 => {
                let (r, g, b) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                );
                match (r, g, b) {
                    (Ok(r), Ok(g), Ok(b)) => Color::Srgba(Srgba {
                        red: r as f32 / 255.0,
                        green: g as f32 / 255.0,
                        blue: b as f32 / 255.0,
                        alpha: 1.0,
                    }),
                    _ => {
                        error!("Invalid hex color format: {}", value);
                        Color::default()
                    }
                }
            }
            8 => {
                let (r, g, b, a) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                    u8::from_str_radix(&hex[6..8], 16),
                );
                match (r, g, b, a) {
                    (Ok(r), Ok(g), Ok(b), Ok(a)) => Color::Srgba(Srgba {
                        red: r as f32 / 255.0,
                        green: g as f32 / 255.0,
                        blue: b as f32 / 255.0,
                        alpha: a as f32 / 255.0,
                    }),
                    _ => {
                        error!("Invalid hex color format: {}", value);
                        Color::default()
                    }
                }
            }
            _ => {
                error!("Unexpected hex color length: {}", hex.len());
                Color::default()
            }
        };
    }

    let value_lower = value.to_ascii_lowercase();
    if value_lower.starts_with("rgb(") || value_lower.starts_with("rgba(") {
        let parts: Vec<_> = value_lower
            .trim_start_matches("rgba(")
            .trim_start_matches("rgb(")
            .trim_end_matches(')')
            .split(',')
            .map(|s| s.trim())
            .collect();

        match parts.as_slice() {
            [r, g, b] => {
                let (r, g, b) = (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>());
                match (r, g, b) {
                    (Ok(r), Ok(g), Ok(b)) => Color::Srgba(Srgba {
                        red: r as f32 / 255.0,
                        green: g as f32 / 255.0,
                        blue: b as f32 / 255.0,
                        alpha: 1.0,
                    }),
                    _ => {
                        error!("Invalid rgb color values: {}", value);
                        Color::default()
                    }
                }
            }
            [r, g, b, a] => {
                let (r, g, b) = (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>());
                let a = a.parse::<f32>();
                match (r, g, b, a) {
                    (Ok(r), Ok(g), Ok(b), Ok(a)) => Color::Srgba(Srgba {
                        red: r as f32 / 255.0,
                        green: g as f32 / 255.0,
                        blue: b as f32 / 255.0,
                        alpha: a.clamp(0.0, 1.0),
                    }),
                    _ => {
                        error!("Invalid rgba color values: {}", value);
                        Color::default()
                    }
                }
            }
            _ => {
                error!("Invalid rgb/rgba format: {}", value);
                Color::default()
            }
        }
    } else {
        match value {
            "White" => Color::WHITE,
            "Black" => Color::BLACK,
            "Red"   => Color::srgb(1.0, 0.0, 0.0),
            "Green" => Color::srgb(0.0, 1.0, 0.0),
            "Blue"  => Color::srgb(0.0, 0.0, 1.0),
            _ => {
                error!("Unsupported color format: {}", value);
                Color::default()
            }
        }
    }
}
