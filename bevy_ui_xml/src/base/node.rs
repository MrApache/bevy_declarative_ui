use std::str::FromStr;
use bevy::prelude::*;
use crate::injector::Injector;
use crate::prelude::{Extractor, FromStrTyped, ValueStorage};
use crate::xml_component::XmlComponent;

pub struct NodeInjector;
impl Injector for NodeInjector {
    fn inject_value(&self, name: &str, value: &ValueStorage, extractor: &mut Extractor, _: &AssetServer) {
        extractor.extract::<Node, _>(|node| set_value_safe(node, name, value));
    }
}

#[derive(Default, Debug, Clone)]
pub struct NodeParser {
    node: Node
}

fn set_value(node: &mut Node, name:&str, value:&str) -> bool {
    match name {
        "overflow_x"               => node.overflow.x = OverflowAxis::from_str_typed(value).unwrap(),
        "overflow_y"               => node.overflow.y = OverflowAxis::from_str_typed(value).unwrap(),
        "overflow_clip_visual_box" => node.overflow_clip_margin.visual_box = OverflowClipBox::from_str_typed(value).unwrap(),
        "overflow_clip_margin"     => node.overflow_clip_margin.margin     = f32::from_str(value).unwrap(),
        "display"                  => node.display         = Display::from_str_typed(value).unwrap(),
        "box_sizing"               => node.box_sizing      = BoxSizing::from_str_typed(value).unwrap(),
        "position_type"            => node.position_type   = PositionType::from_str_typed(value).unwrap(),
        "left"                     => node.left            = Val::from_str_typed(value).unwrap(),
        "right"                    => node.right           = Val::from_str_typed(value).unwrap(),
        "top"                      => node.top             = Val::from_str_typed(value).unwrap(),
        "bottom"                   => node.bottom          = Val::from_str_typed(value).unwrap(),
        "width"                    => node.width           = Val::from_str_typed(value).unwrap(),
        "height"                   => node.height          = Val::from_str_typed(value).unwrap(),
        "min_width"                => node.min_width       = Val::from_str_typed(value).unwrap(),
        "min_height"               => node.min_height      = Val::from_str_typed(value).unwrap(),
        "max_width"                => node.max_width       = Val::from_str_typed(value).unwrap(),
        "max_height"               => node.max_height      = Val::from_str_typed(value).unwrap(),
        "aspect_ratio"             => node.aspect_ratio    = f32::from_str(value).ok(),
        "align_items"              => node.align_items     = AlignItems::from_str_typed(value).unwrap(),
        "justify_items"            => node.justify_items   = JustifyItems::from_str_typed(value).unwrap(),
        "align_self"               => node.align_self      = AlignSelf::from_str_typed(value).unwrap(),
        "justify_self"             => node.justify_self    = JustifySelf::from_str_typed(value).unwrap(),
        "align_content"            => node.align_content   = AlignContent::from_str_typed(value).unwrap(),
        "justify_content"          => node.justify_content = JustifyContent::from_str_typed(value).unwrap(),
        "margin"                   => node.margin          = UiRect::from_str_typed(value).unwrap(),
        "padding"                  => node.padding         = UiRect::from_str_typed(value).unwrap(),
        "border"                   => node.border          = UiRect::from_str_typed(value).unwrap(),
        "flex_direction"           => node.flex_direction  = FlexDirection::from_str_typed(value).unwrap(),
        "flex_wrap"                => node.flex_wrap       = FlexWrap::from_str_typed(value).unwrap(),
        "flex_grow"                => node.flex_grow       = f32::from_str(value).unwrap(),
        "flex_shrink"              => node.flex_shrink     = f32::from_str(value).unwrap(),
        "flex_basis"               => node.flex_basis      = Val::from_str_typed(value).unwrap(),
        "row_gap"                  => node.row_gap         = Val::from_str_typed(value).unwrap(),
        "column_gap"               => node.column_gap      = Val::from_str_typed(value).unwrap(),
        _ => return false,
    }

    true
}

fn set_value_safe(node: &mut Node, name:&str, value: &ValueStorage) {
    match name {
        "overflow_clip_visual_box" => node.overflow_clip_margin.visual_box = *value.read::<OverflowClipBox>(),
        "overflow_clip_margin"     => node.overflow_clip_margin.margin     = *value.read::<f32>(),
        "overflow_x"               => node.overflow.x      = *value.read::<OverflowAxis>(),
        "overflow_y"               => node.overflow.y      = *value.read::<OverflowAxis>(),
        "display"                  => node.display         = *value.read::<Display>(),
        "box_sizing"               => node.box_sizing      = *value.read::<BoxSizing>(),
        "position_type"            => node.position_type   = *value.read::<PositionType>(),

        "left"                     => node.left            = *value.read::<Val>(),
        "right"                    => node.right           = *value.read::<Val>(),
        "top"                      => node.top             = *value.read::<Val>(),
        "bottom"                   => node.bottom          = *value.read::<Val>(),
        "width"                    => node.width           = *value.read::<Val>(),
        "height"                   => node.height          = *value.read::<Val>(),
        "min_width"                => node.min_width       = *value.read::<Val>(),
        "min_height"               => node.min_height      = *value.read::<Val>(),
        "max_width"                => node.max_width       = *value.read::<Val>(),
        "max_height"               => node.max_height      = *value.read::<Val>(),
        "aspect_ratio"             => node.aspect_ratio    = *value.read::<Option<f32>>(),

        "align_items"              => node.align_items     = *value.read::<AlignItems>(),
        "justify_items"            => node.justify_items   = *value.read::<JustifyItems>(),
        "align_self"               => node.align_self      = *value.read::<AlignSelf>(),
        "justify_self"             => node.justify_self    = *value.read::<JustifySelf>(),
        "align_content"            => node.align_content   = *value.read::<AlignContent>(),
        "justify_content"          => node.justify_content = *value.read::<JustifyContent>(),

        "margin"                   => node.margin          = *value.read::<UiRect>(),
        "padding"                  => node.padding         = *value.read::<UiRect>(),
        "border"                   => node.border          = *value.read::<UiRect>(),
        "flex_direction"           => node.flex_direction  = *value.read::<FlexDirection>(),
        "flex_wrap"                => node.flex_wrap       = *value.read::<FlexWrap>(),
        "flex_grow"                => node.flex_grow       = *value.read::<f32>(),
        "flex_shrink"              => node.flex_shrink     = *value.read::<f32>(),
        "flex_basis"               => node.flex_basis      = *value.read::<Val>(),
        "row_gap"                  => node.row_gap         = *value.read::<Val>(),
        "column_gap"               => node.column_gap      = *value.read::<Val>(),
        _ => {},
    }
}

impl XmlComponent for NodeParser {
    fn write_value(&mut self, name: &str, value: &ValueStorage) {
        set_value_safe(&mut self.node, name, value)
    }

    fn insert_to(&self, entity: &mut EntityCommands, _: &AssetServer) {
        entity.insert(self.node.clone());
    }

    fn as_injector(&self) -> Box<dyn Injector> {
        Box::new(NodeInjector)
    }

    fn parse_attribute(&mut self, name: &str, value: &str) -> bool {
        set_value(&mut self.node, name, value)
    }
}