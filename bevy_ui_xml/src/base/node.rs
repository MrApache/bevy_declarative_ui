use std::str::FromStr;
use bevy::prelude::*;
use crate::types::parse_flex_direction;
use crate::xml_component::XmlComponent;

#[derive(Default, Debug, Clone)]
pub struct NodeParser {
    node: Node
}

impl XmlComponent for NodeParser {
    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(self.node.clone());
    }

    fn clear(&mut self) {
        self.node = Node::default();
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        match name {
            "overflow_x"               => self.node.overflow.x = parse_overflow_axis(value),
            "overflow_y"               => self.node.overflow.y = parse_overflow_axis(value),
            "overflow_clip_visual_box" => self.node.overflow_clip_margin.visual_box = parse_overflow_clip_visual_box(value),
            "overflow_clip_margin"     => self.node.overflow_clip_margin.margin     = f32::from_str(value).unwrap(),
            "display"         => self.node.display         = parse_display(value),
            "box_sizing"      => self.node.box_sizing      = parse_box_sizing(value),
            "position_type"   => self.node.position_type   = parse_position_type(value),
            "left"            => self.node.left            = parse_val(value),
            "right"           => self.node.right           = parse_val(value),
            "top"             => self.node.top             = parse_val(value),
            "bottom"          => self.node.bottom          = parse_val(value),
            "width"           => self.node.width           = parse_val(value),
            "height"          => self.node.height          = parse_val(value),
            "min_width"       => self.node.min_width       = parse_val(value),
            "min_height"      => self.node.min_height      = parse_val(value),
            "max_width"       => self.node.max_width       = parse_val(value),
            "max_height"      => self.node.max_height      = parse_val(value),
            "aspect_ratio"    => self.node.aspect_ratio    = f32::from_str(value).ok(),
            "align_items"     => self.node.align_items     = parse_align_items(value),
            "justify_items"   => self.node.justify_items   = parse_justify_items(value),
            "align_self"      => self.node.align_self      = parse_align_self(value),
            "justify_self"    => self.node.justify_self    = parse_justify_self(value),
            "align_content"   => self.node.align_content   = parse_align_content(value),
            "justify_content" => self.node.justify_content = parse_justify_content(value),
            "margin"          => self.node.margin          = parse_ui_rect(value),
            "padding"         => self.node.padding         = parse_ui_rect(value),
            "border"          => self.node.border          = parse_ui_rect(value),
            "flex_direction"  => self.node.flex_direction  = parse_flex_direction(value),
            "flex_wrap"       => self.node.flex_wrap       = parse_flex_wrap(value),
            "flex_grow"       => self.node.flex_grow       = f32::from_str(value).unwrap(),
            "flex_shrink"     => self.node.flex_shrink     = f32::from_str(value).unwrap(),
            "flex_basis"      => self.node.flex_basis      = parse_val(value),
            "row_gap"         => self.node.row_gap         = parse_val(value),
            "column_gap"      => self.node.column_gap      = parse_val(value),
            _ => return false
        }

        true
    }
}

fn parse_display(str: &str) -> Display {
    match str {
        "Flex"  => Display::Flex,
        "None"  => Display::None,
        "Block" => Display::Block,
        "Grid"  => Display::Grid,
        _ => {
            error!("Unknown display value: {}", str);
            Display::default()
        }
    }
}

fn parse_box_sizing(str: &str) -> BoxSizing {
    match str {
        "BorderBox"  => BoxSizing::BorderBox,
        "ContentBox" => BoxSizing::ContentBox,
        _ => {
            error!("Unknown box sizing value: {}", str);
            BoxSizing::default()
        }
    }
}

fn parse_position_type(str: &str) -> PositionType {
    match str {
        "Absolute" => PositionType::Absolute,
        "Relative" => PositionType::Relative,
        _ => {
            error!("Unknown position type value: {}", str);
            PositionType::default()
        }
    }
}

fn parse_overflow_axis(str: &str) -> OverflowAxis {
    match str {
        "Visible" => OverflowAxis::Visible,
        "Hidden"  => OverflowAxis::Hidden,
        "Scroll"  => OverflowAxis::Scroll,
        "Clip"    => OverflowAxis::Clip,
        _ => {
            error!("Unknown overflow axis value: {}", str);
            OverflowAxis::default()
        }
    }
}

fn parse_overflow_clip_visual_box(str: &str) -> OverflowClipBox {
    match str {
        "ContentBox" => OverflowClipBox::ContentBox,
        "PaddingBox" => OverflowClipBox::PaddingBox,
        "BorderBox"  => OverflowClipBox::BorderBox,
        _ => {
            error!("Unknown overflow clip visual box value: {}", str);
            OverflowClipBox::default()
        }
    }
}

fn parse_ui_rect(input: &str) -> UiRect {
    let mut result = Vec::new();

    for word in input.split_whitespace() {
        result.push(parse_val(word));
    }

    match result.len() {
        1 => UiRect::all(result[0].clone()),
        2 => UiRect::new(result[0].clone(), result[1].clone(), result[0].clone(), result[1].clone()),
        3 => UiRect::new(result[0].clone(), result[1].clone(), result[2].clone(), result[1].clone()),
        4 => UiRect::new(result[0].clone(), result[1].clone(), result[2].clone(), result[3].clone()),
        _ => panic!("todo"),
    }
}

fn parse_val(input: &str) -> Val {
    if input == "auto" {
        Val::Auto
    } else if let Some(px) = input.strip_suffix("px") {
        return px.parse().ok().map(Val::Px).unwrap();
    } else if let Some(pc) = input.strip_suffix('%') {
        return pc.parse().ok().map(Val::Percent).unwrap();
    } else if let Some(inner) = input.strip_prefix("vw(").and_then(|s| s.strip_suffix(")")) {
        return inner.parse().ok().map(Val::Vw).unwrap();
    } else if let Some(inner) = input.strip_prefix("vh(").and_then(|s| s.strip_suffix(")")) {
        return inner.parse().ok().map(Val::Vh).unwrap();
    } else if let Some(inner) = input.strip_prefix("vmin(").and_then(|s| s.strip_suffix(")")) {
        return inner.parse().ok().map(Val::VMin).unwrap();
    } else if let Some(inner) = input.strip_prefix("vmax(").and_then(|s| s.strip_suffix(")")) {
        return inner.parse().ok().map(Val::VMax).unwrap();
    }
    else {
        error!("Unknown value: {}", input);
        Val::default()
    }
}

fn parse_align_items(str: &str) -> AlignItems {
    match str {
        "Default"   => AlignItems::Default,
        "Start"     => AlignItems::Start,
        "End"       => AlignItems::End,
        "FlexStart" => AlignItems::FlexStart,
        "FlexEnd"   => AlignItems::FlexEnd,
        "Center"    => AlignItems::Center,
        "Baseline"  => AlignItems::Baseline,
        "Stretch"   => AlignItems::Stretch,
        _ => {
            error!("Unknown align items value: {}", str);
            AlignItems::default()
        }
    }
}

fn parse_justify_items(str: &str) -> JustifyItems {
    match str {
        "Default"  => JustifyItems::Default,
        "Start"    => JustifyItems::Start,
        "End"      => JustifyItems::End,
        "Center"   => JustifyItems::Center,
        "Baseline" => JustifyItems::Baseline,
        "Stretch"  => JustifyItems::Stretch,
        _ => {
            error!("Unknown justify items value: {}", str);
            JustifyItems::default()
        }
    }
}

fn parse_align_self(str: &str) -> AlignSelf {
    match str {
        "Auto"      => AlignSelf::Auto,
        "Start"     => AlignSelf::Start,
        "End"       => AlignSelf::End,
        "FlexStart" => AlignSelf::FlexStart,
        "FlexEnd"   => AlignSelf::FlexEnd,
        "Center"    => AlignSelf::Center,
        "Baseline"  => AlignSelf::Baseline,
        "Stretch"   => AlignSelf::Stretch,
        _ => {
            error!("Unknown align self value: {}", str);
            AlignSelf::default()
        }
    }
}

fn parse_justify_self(str: &str) -> JustifySelf {
    match str {
        "Auto"     => JustifySelf::Auto,
        "Start"    => JustifySelf::Start,
        "End"      => JustifySelf::End,
        "Center"   => JustifySelf::Center,
        "Baseline" => JustifySelf::Baseline,
        "Stretch"  => JustifySelf::Stretch,
        _ =>  {
            error!("Unknown justify self value: {}", str);
            JustifySelf::default()
        }
    }
}

fn parse_align_content(str: &str) -> AlignContent {
    match str {
        "Default"      => AlignContent::Default,
        "Start"        => AlignContent::Start,
        "End"          => AlignContent::End,
        "FlexStart"    => AlignContent::FlexStart,
        "FlexEnd"      => AlignContent::FlexEnd,
        "Center"       => AlignContent::Center,
        "Stretch"      => AlignContent::Stretch,
        "SpaceBetween" => AlignContent::SpaceBetween,
        "SpaceEvenly"  => AlignContent::SpaceEvenly,
        "SpaceAround"  => AlignContent::SpaceAround,
        _ => {
            error!("Unknown align content value: {}", str);
            AlignContent::Default
        }
    }
}

fn parse_justify_content(str: &str) -> JustifyContent {
    match str {
        "Default"      => JustifyContent::Default,
        "Start"        => JustifyContent::Start,
        "End"          => JustifyContent::End,
        "FlexStart"    => JustifyContent::FlexStart,
        "FlexEnd"      => JustifyContent::FlexEnd,
        "Center"       => JustifyContent::Center,
        "Stretch"      => JustifyContent::Stretch,
        "SpaceBetween" => JustifyContent::SpaceBetween,
        "SpaceEvenly"  => JustifyContent::SpaceEvenly,
        "SpaceAround"  => JustifyContent::SpaceAround,
        _ => {
            error!("Unknown justify content value: {}", str);
            JustifyContent::default()
        }
    }
}

fn parse_flex_wrap(str: &str) -> FlexWrap {
    match str {
        "NoWrap"      => FlexWrap::NoWrap,
        "Wrap"        => FlexWrap::Wrap,
        "WrapReverse" => FlexWrap::WrapReverse,
        _ => {
            error!("Unknown flex wrap value: {}", str);
            FlexWrap::default()
        }
    }
}
