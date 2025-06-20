mod text_color;
mod node;
mod background_color;
mod text_font;
mod text_layout;
mod text;
mod image;
mod button;

use std::str::FromStr;
use bevy::prelude::*;
use bevy::text::{FontSmoothing, LineHeight};
pub use node::{NodeParser, NodeInjector};
pub use background_color::BackgroundColorParser;
pub use text_color::{TextColorParser, TextColorInjector};
pub use text_font::{TextFontParser, TextFontInjector};
pub use text_layout::TextLayoutParser;
pub use text::{TextParser, TextInjector};
pub use image::{ImageNodeParser, ImageNodeInjector};
pub use button::ButtonParser;
use crate::prelude::{FromStrTyped, MutValueStorage, XmlLibrary};

pub fn add_base(library: &mut XmlLibrary) {
    library.add_component("BackgroundColor", || Box::new(BackgroundColorParser::default()));
    library.add_component("Node",            || Box::new(NodeParser::default()));
    library.add_component("ImageNode",       || Box::new(ImageNodeParser::default()));
    library.add_component("TextFont",        || Box::new(TextFontParser::default()));
    library.add_component("TextColor",       || Box::new(TextColorParser::default()));
    library.add_component("TextLayout",      || Box::new(TextLayoutParser::default()));
    library.add_component("Text",            || Box::new(TextParser::default()));
    library.add_component("Button",          || Box::new(ButtonParser));
}

pub fn add_base_types(library: &mut XmlLibrary) {
    library.add_type::<i8>("i8");
    library.add_type::<i16>("i16");
    library.add_type::<i32>("i32");
    library.add_type::<i64>("i64");

    library.add_type::<u8>("u8");
    library.add_type::<u16>("u16");
    library.add_type::<u32>("u32");
    library.add_type::<u64>("u64");

    library.add_type::<f32>("f32");
    library.add_type::<f64>("f64");

    library.add_type::<char>("char");
    library.add_type::<bool>("bool");
    library.add_type::<String>("String");

    library.add_type::<Display>("Display");
    library.add_type::<BoxSizing>("BoxSizing");
    library.add_type::<PositionType>("PositionType");
    library.add_type::<OverflowAxis>("OverflowAxis");
    library.add_type::<OverflowClipBox>("OverflowClipBox");
    library.add_type::<Val>("Val");
    library.add_type::<UiRect>("UiRect");
    library.add_type::<AlignItems>("AlignItems");
    library.add_type::<JustifyItems>("JustifyItems");
    library.add_type::<AlignSelf>("AlignSelf");
    library.add_type::<JustifySelf>("JustifySelf");
    library.add_type::<AlignContent>("AlignContent");
    library.add_type::<JustifyContent>("JustifyContent");
    library.add_type::<FlexWrap>("FlexWrap");
    library.add_type::<FlexDirection>("FlexDirection");

    library.add_type::<Color>("Color");
}

macro_rules! impl_from_str_typed_std {
    ($type:ty) => {
        impl $crate::IsTyped for $type {
            fn write_to_storage(&self, value: &str, storage: &mut MutValueStorage) {
                storage.write::<Self>(value);
            }
        }

        impl $crate::FromStrTyped for $type {
            fn from_str_typed(s: &str) -> Result<Self, String> {
                <$type as std::str::FromStr>::from_str(s).map_err(|e| e.to_string())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_is_typed {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $crate::prelude::IsTyped for $ty {
                fn write_to_storage(&self, value: &str, storage: &mut $crate::prelude::MutValueStorage) {
                    storage.write::<Self>(value);
                }
            }
        )*
    };
}

impl_from_str_typed_std!(i8);
impl_from_str_typed_std!(i16);
impl_from_str_typed_std!(i32);
impl_from_str_typed_std!(i64);

impl_from_str_typed_std!(u8);
impl_from_str_typed_std!(u16);
impl_from_str_typed_std!(u32);
impl_from_str_typed_std!(u64);

impl_from_str_typed_std!(f32);
impl_from_str_typed_std!(f64);

impl_from_str_typed_std!(String);
impl_from_str_typed_std!(bool);
impl_from_str_typed_std!(char);

impl_is_typed!(Display);
impl FromStrTyped for Display {
    fn from_str_typed(s: &str) -> Result<Self, String>
    {
        match s {
            "Flex"  => Ok(Display::Flex),
            "None"  => Ok(Display::None),
            "Block" => Ok(Display::Block),
            "Grid"  => Ok(Display::Grid),
            _ => Err(format!("Unknown display value: {}", s))
        }
    }
}

impl_is_typed!(BoxSizing);
impl FromStrTyped for BoxSizing {
    fn from_str_typed(s: &str) -> Result<Self, String>
    {
        match s {
            "BorderBox"  => Ok(BoxSizing::BorderBox),
            "ContentBox" => Ok(BoxSizing::ContentBox),
            _ => Err(format!("Unknown box sizing value: {}", s))
        }
    }
}

impl_is_typed!(PositionType);
impl FromStrTyped for PositionType {
    fn from_str_typed(s: &str) -> Result<PositionType, String> {
        match s {
            "Absolute" => Ok(PositionType::Absolute),
            "Relative" => Ok(PositionType::Relative),
            _ => Err(format!("Unknown position type: {}", s))
        }
    }
}

impl_is_typed!(OverflowAxis);
impl FromStrTyped for OverflowAxis {
    fn from_str_typed(s: &str) -> Result<OverflowAxis, String> {
        match s {
            "Visible" => Ok(OverflowAxis::Visible),
            "Hidden"  => Ok(OverflowAxis::Hidden),
            "Scroll"  => Ok(OverflowAxis::Scroll),
            "Clip"    => Ok(OverflowAxis::Clip),
            _ => Err(format!("Unknown overflow axis: {}", s))
        }
    }
}

impl_is_typed!(OverflowClipBox);
impl FromStrTyped for OverflowClipBox {
    fn from_str_typed(s: &str) -> Result<OverflowClipBox, String> {
        match s {
            "ContentBox" => Ok(OverflowClipBox::ContentBox),
            "PaddingBox" => Ok(OverflowClipBox::PaddingBox),
            "BorderBox"  => Ok(OverflowClipBox::BorderBox),
            _ => Err(format!("Unknown overflow clip box value: {}", s))
        }
    }
}

impl_is_typed!(Val);
impl FromStrTyped for Val{
    fn from_str_typed(s: &str) -> Result<Val, String> {
        if s == "auto" {
            Ok(Val::Auto)
        } else if let Some(px) = s.strip_suffix("px") {
            return Ok(px.parse().ok().map(Val::Px).unwrap());
        } else if let Some(pc) = s.strip_suffix('%') {
            return Ok(pc.parse().ok().map(Val::Percent).unwrap());
        } else if let Some(inner) = s.strip_prefix("vw(").and_then(|s| s.strip_suffix(")")) {
            return Ok(inner.parse().ok().map(Val::Vw).unwrap());
        } else if let Some(inner) = s.strip_prefix("vh(").and_then(|s| s.strip_suffix(")")) {
            return Ok(inner.parse().ok().map(Val::Vh).unwrap());
        } else if let Some(inner) = s.strip_prefix("vmin(").and_then(|s| s.strip_suffix(")")) {
            return Ok(inner.parse().ok().map(Val::VMin).unwrap());
        } else if let Some(inner) = s.strip_prefix("vmax(").and_then(|s| s.strip_suffix(")")) {
            return Ok(inner.parse().ok().map(Val::VMax).unwrap());
        }
        else {
            Err(format!("Unknown value: {}", s))
        }
    }
}

impl_is_typed!(UiRect);
impl FromStrTyped for UiRect {
    fn from_str_typed(s: &str) -> Result<UiRect, String> {
        let mut result: Vec<Result<Val, String>> = Vec::new();

        for word in s.split_whitespace() {
            result.push(Val::from_str_typed(word));
        }

        match result.len() {
            1 => Ok(UiRect::all(result[0].clone()?)),
            2 => Ok(UiRect::new(result[0].clone()?, result[1].clone()?, result[0].clone()?, result[1].clone()?)),
            3 => Ok(UiRect::new(result[0].clone()?, result[1].clone()?, result[2].clone()?, result[1].clone()?)),
            4 => Ok(UiRect::new(result[0].clone()?, result[1].clone()?, result[2].clone()?, result[3].clone()?)),
            _ => panic!("todo"),
        }
    }
}

impl_is_typed!(AlignItems);
impl FromStrTyped for AlignItems {
    fn from_str_typed(s: &str) -> Result<AlignItems, String> {
        match s {
            "Default"   => Ok(AlignItems::Default),
            "Start"     => Ok(AlignItems::Start),
            "End"       => Ok(AlignItems::End),
            "FlexStart" => Ok(AlignItems::FlexStart),
            "FlexEnd"   => Ok(AlignItems::FlexEnd),
            "Center"    => Ok(AlignItems::Center),
            "Baseline"  => Ok(AlignItems::Baseline),
            "Stretch"   => Ok(AlignItems::Stretch),
            _ => Err(format!("[AlignItems] Unknown value: {}", s))
        }
    }
}

impl_is_typed!(JustifyItems);
impl FromStrTyped for JustifyItems {
    fn from_str_typed(s: &str) -> Result<JustifyItems, String> {
        match s {
            "Default"  => Ok(JustifyItems::Default),
            "Start"    => Ok(JustifyItems::Start),
            "End"      => Ok(JustifyItems::End),
            "Center"   => Ok(JustifyItems::Center),
            "Baseline" => Ok(JustifyItems::Baseline),
            "Stretch"  => Ok(JustifyItems::Stretch),
            _ => Err(format!("[JustifyItems] Unknown value: {}", s))
        }
    }
}

impl_is_typed!(AlignSelf);
impl FromStrTyped for AlignSelf {
    fn from_str_typed(s: &str) -> Result<AlignSelf, String> {
        match s {
            "Auto"      => Ok(AlignSelf::Auto),
            "Start"     => Ok(AlignSelf::Start),
            "End"       => Ok(AlignSelf::End),
            "FlexStart" => Ok(AlignSelf::FlexStart),
            "FlexEnd"   => Ok(AlignSelf::FlexEnd),
            "Center"    => Ok(AlignSelf::Center),
            "Baseline"  => Ok(AlignSelf::Baseline),
            "Stretch"   => Ok(AlignSelf::Stretch),
            _ => Err(format!("Unknown alignment value: {}", s))
        }
    }
}

impl_is_typed!(JustifySelf);
impl FromStrTyped for JustifySelf {
    fn from_str_typed(s: &str) -> Result<JustifySelf, String> {
        match s {
            "Auto"     => Ok(JustifySelf::Auto),
            "Start"    => Ok(JustifySelf::Start),
            "End"      => Ok(JustifySelf::End),
            "Center"   => Ok(JustifySelf::Center),
            "Baseline" => Ok(JustifySelf::Baseline),
            "Stretch"  => Ok(JustifySelf::Stretch),
            _ => Err(format!("Unknown justify value: {}", s))
        }
    }
}

impl_is_typed!(AlignContent);
impl FromStrTyped for AlignContent {
    fn from_str_typed(s: &str) -> Result<AlignContent, String> {
        match s {
            "Default"      => Ok(AlignContent::Default),
            "Start"        => Ok(AlignContent::Start),
            "End"          => Ok(AlignContent::End),
            "FlexStart"    => Ok(AlignContent::FlexStart),
            "FlexEnd"      => Ok(AlignContent::FlexEnd),
            "Center"       => Ok(AlignContent::Center),
            "Stretch"      => Ok(AlignContent::Stretch),
            "SpaceBetween" => Ok(AlignContent::SpaceBetween),
            "SpaceEvenly"  => Ok(AlignContent::SpaceEvenly),
            "SpaceAround"  => Ok(AlignContent::SpaceAround),
            _ => Err(format!("Unknown alignment value: {}", s))
        }
    }
}

impl_is_typed!(JustifyContent);
impl FromStrTyped for JustifyContent {
    fn from_str_typed(s: &str) -> Result<JustifyContent, String> {
        match s {
            "Default"      => Ok(JustifyContent::Default),
            "Start"        => Ok(JustifyContent::Start),
            "End"          => Ok(JustifyContent::End),
            "FlexStart"    => Ok(JustifyContent::FlexStart),
            "FlexEnd"      => Ok(JustifyContent::FlexEnd),
            "Center"       => Ok(JustifyContent::Center),
            "Stretch"      => Ok(JustifyContent::Stretch),
            "SpaceBetween" => Ok(JustifyContent::SpaceBetween),
            "SpaceEvenly"  => Ok(JustifyContent::SpaceEvenly),
            "SpaceAround"  => Ok(JustifyContent::SpaceAround),
            _ => Err(format!("Unknown justify value: {}", s))
        }
    }
}

impl_is_typed!(FlexWrap);
impl FromStrTyped for FlexWrap {
    fn from_str_typed(s: &str) -> Result<FlexWrap, String> {
        match s {
            "NoWrap"      => Ok(FlexWrap::NoWrap),
            "Wrap"        => Ok(FlexWrap::Wrap),
            "WrapReverse" => Ok(FlexWrap::WrapReverse),
            _ => Err(format!("Unknown flex wrap value: {}", s))
        }
    }
}

impl_is_typed!(FlexDirection);
impl FromStrTyped for FlexDirection {
    fn from_str_typed(s: &str) -> Result<Self, String> {
        match s {
            "Row"           => Ok(FlexDirection::Row),
            "Column"        => Ok(FlexDirection::Column),
            "RowReverse"    => Ok(FlexDirection::RowReverse),
            "ColumnReverse" => Ok(FlexDirection::ColumnReverse),
            _ => Err(format!("Unknown flex direction value: {}", s))
        }
    }
}

impl_is_typed!(Color);
impl FromStrTyped for Color {
    fn from_str_typed(s: &str) -> Result<Self, String> {
        let value: &str = s.trim();

        if let Some(hex) = value.strip_prefix('#') {
            return match hex.len() {
                6 => {
                    let (r, g, b) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    );
                    match (r, g, b) {
                        (Ok(r), Ok(g), Ok(b)) => Ok(Color::Srgba(Srgba {
                            red: r as f32 / 255.0,
                            green: g as f32 / 255.0,
                            blue: b as f32 / 255.0,
                            alpha: 1.0,
                        })),
                        _ => Err(format!("Invalid hex color format: {}", value))
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
                        (Ok(r), Ok(g), Ok(b), Ok(a)) => Ok(Color::Srgba(Srgba {
                            red: r as f32 / 255.0,
                            green: g as f32 / 255.0,
                            blue: b as f32 / 255.0,
                            alpha: a as f32 / 255.0,
                        })),
                        _ => Err(format!("Invalid hex color format: {}", value))
                    }
                }
                _ => Err(format!("Unexpected hex color length: {}", hex.len()))
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
                        (Ok(r), Ok(g), Ok(b)) => Ok(Color::Srgba(Srgba {
                            red: r as f32 / 255.0,
                            green: g as f32 / 255.0,
                            blue: b as f32 / 255.0,
                            alpha: 1.0,
                        })),
                        _ => Err(format!("Invalid rgb color values: {}", value))
                    }
                }
                [r, g, b, a] => {
                    let (r, g, b) = (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>());
                    let a = a.parse::<f32>();
                    match (r, g, b, a) {
                        (Ok(r), Ok(g), Ok(b), Ok(a)) => Ok(Color::Srgba(Srgba {
                            red: r as f32 / 255.0,
                            green: g as f32 / 255.0,
                            blue: b as f32 / 255.0,
                            alpha: a.clamp(0.0, 1.0),
                        })),
                        _ => Err(format!("Invalid rgba color values: {}", value))
                    }
                }
                _ => Err(format!("Invalid rgb/rgba format: {}", value))
            }
        } else {
            match value {
                "White" => Ok(Color::WHITE),
                "Black" => Ok(Color::BLACK),
                "Red"   => Ok(Color::srgb(1.0, 0.0, 0.0)),
                "Green" => Ok(Color::srgb(0.0, 1.0, 0.0)),
                "Blue"  => Ok(Color::srgb(0.0, 0.0, 1.0)),
                _ => Err(format!("Unknown color value: {}", value))
            }
        }
    }
}

impl_is_typed!(JustifyText);
impl FromStrTyped for JustifyText {
    fn from_str_typed(s: &str) -> Result<Self, String> {
        match s {
            "Left"      => Ok(JustifyText::Left),
            "Center"    => Ok(JustifyText::Center),
            "Right"     => Ok(JustifyText::Right),
            "Justified" => Ok(JustifyText::Justified),
            _ => Err(format!("[TextLayout] Unknown value: {}", s))
        }
    }
}

impl_is_typed!(LineBreak);

impl FromStrTyped for LineBreak {
    fn from_str_typed(s: &str) -> Result<Self, String> {
        match s {
            "WordBoundary"    => Ok(LineBreak::WordBoundary),
            "AnyCharacter"    => Ok(LineBreak::AnyCharacter),
            "WordOrCharacter" => Ok(LineBreak::WordOrCharacter),
            "NoWrap"          => Ok(LineBreak::NoWrap),
            _ => Err(format!("[LineBreak] Unknown value: {}", s))
        }
    }
}

impl_is_typed!(LineHeight);
impl FromStrTyped for LineHeight {
    fn from_str_typed(s: &str) -> Result<Self, String> {
        let s = s.trim();

        if let Some(num_str) = s.strip_suffix("px") {
            match f32::from_str(num_str.trim()) {
                Ok(value) => Ok(LineHeight::Px(value)),
                Err(_) => Err(format!("Failed to parse number before `px`: {:?}", num_str)),
            }
        } else if let Some(num_str) = s.strip_suffix("rl") {
            match f32::from_str(num_str.trim()) {
                Ok(value) => Ok(LineHeight::RelativeToFont(value)),
                Err(_) => Err(format!("Failed to parse number before `rl`: {:?}", num_str))
            }
        } else {
            Err(format!("String `{}` does not match the pattern `<number>px` or `<number>rl`", s))
        }
    }
}

impl_is_typed!(FontSmoothing);
impl FromStrTyped for FontSmoothing {
    fn from_str_typed(s: &str) -> Result<Self, String> {
        match s {
            "None"        => Ok(FontSmoothing::None),
            "AntiAliased" => Ok(FontSmoothing::AntiAliased),
            _ => Err(format!("[FontSmoothing] Unknown value: {}", s))
        }
    }
}