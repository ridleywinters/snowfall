use bevy::{log, prelude::*};
use regex::Regex;
use std::sync::LazyLock;

pub trait EntityCommandsUIExt {
    fn style(self, style: &str) -> Self;
    fn styles(self, styles: &Vec<&str>) -> Self;
    fn text(self, content: &str) -> Self;
}

impl<'a> EntityCommandsUIExt for EntityCommands<'a> {
    fn style(mut self, style: &str) -> Self {
        node_style(&mut self, style);
        self
    }
    fn styles(mut self, styles: &Vec<&str>) -> Self {
        node_style(&mut self, styles.join(" ").as_str());
        self
    }
    fn text(mut self, content: &str) -> Self {
        self.insert(Text::new(content));
        self
    }
}

#[derive(Default)]
struct StyledBundle {
    node: Node,
    z_index: Option<ZIndex>,
    background_color: Option<BackgroundColor>,
    text_font: Option<TextFont>,
    text_color: Option<TextColor>,
    outline: Option<Outline>,
}

enum StyleHandler {
    Void(fn(&mut StyledBundle)),
    I32(fn(&mut StyledBundle, i32)),
    F32(fn(&mut StyledBundle, f32)),
    Str(fn(&mut StyledBundle, &str)),
    F32F32F32(fn(&mut StyledBundle, f32, f32, f32)),
    F32F32F32F32(fn(&mut StyledBundle, f32, f32, f32, f32)),
}

static COMPILED_PATTERNS: LazyLock<Vec<(Regex, StyleHandler)>> = LazyLock::new(|| {
    use StyleHandler::*;
    let patterns: Vec<(&str, StyleHandler)> = vec![
        //
        // Positioning
        //
        (
            "absolute",
            Void(|b| {
                b.node.position_type = PositionType::Absolute;
            }),
        ),
        (
            "relative",
            Void(|b| {
                b.node.position_type = PositionType::Relative;
            }),
        ),
        (
            r"top-(\d+)",
            I32(|b, v| {
                b.node.top = Val::Px(v as f32);
            }),
        ),
        (
            r"left-(\d+)",
            I32(|b, v| {
                b.node.left = Val::Px(v as f32);
            }),
        ),
        (
            r"bottom-(\d+)",
            I32(|b, v| {
                b.node.bottom = Val::Px(v as f32);
            }),
        ),
        (
            r"right-(\d+)",
            I32(|b, v| {
                b.node.right = Val::Px(v as f32);
            }),
        ),
        (
            r"width-(\d+)",
            I32(|b, v| {
                b.node.width = Val::Px(v as f32);
            }),
        ),
        (
            r"width-(\d+)%",
            I32(|b, v| {
                b.node.width = Val::Percent(v as f32);
            }),
        ),
        (
            r"height-(\d+)",
            I32(|b, v| {
                b.node.height = Val::Px(v as f32);
            }),
        ),
        (
            r"height-(\d+)%",
            I32(|b, v| {
                b.node.height = Val::Percent(v as f32);
            }),
        ),
        (
            r"z(\d+)",
            I32(|b, v| {
                b.z_index = Some(ZIndex(v));
            }),
        ),
        //
        // Display
        //
        (
            "display-none",
            Void(|b| {
                b.node.display = Display::None;
            }),
        ),
        //
        // Flex related
        //
        (
            "flex-row",
            Void(|b| {
                b.node.flex_direction = FlexDirection::Row;
            }),
        ),
        (
            "flex-row-center",
            Void(|b| {
                b.node.flex_direction = FlexDirection::Row;
                b.node.align_items = AlignItems::Center;
            }),
        ),
        (
            "flex-col",
            Void(|b| {
                b.node.flex_direction = FlexDirection::Column;
            }),
        ),
        (
            "flex-col-center",
            Void(|b| {
                b.node.flex_direction = FlexDirection::Column;
                b.node.align_items = AlignItems::Center;
                b.node.justify_content = JustifyContent::Center;
            }),
        ),
        (
            r"gap-?(\d+)",
            I32(|b, v| {
                b.node.column_gap = Val::Px(v as f32);
                b.node.row_gap = Val::Px(v as f32);
            }),
        ),
        (
            r"grow-?(\d+)",
            I32(|b, v| {
                b.node.flex_grow = v as f32;
            }),
        ),
        (
            r"align-(start|center|end)",
            Str(|b, v| {
                b.node.align_items = match v {
                    "start" => AlignItems::FlexStart,
                    "center" => AlignItems::Center,
                    "end" => AlignItems::FlexEnd,
                    _ => {
                        log::warn!("Invalid align value: {}", v);
                        AlignItems::FlexStart
                    }
                };
            }),
        ),
        (
            r"justify-(start|center|end)",
            Str(|b, v| {
                b.node.justify_content = match v {
                    "start" => JustifyContent::FlexStart,
                    "center" => JustifyContent::Center,
                    "end" => JustifyContent::FlexEnd,
                    _ => {
                        log::warn!("Invalid justify value: {}", v);
                        JustifyContent::FlexStart
                    }
                };
            }),
        ),
        //
        // Overflow
        //
        (
            "scroll-y",
            Void(|b| {
                b.node.overflow = Overflow::scroll_y();
            }),
        ),
        //
        // Margins
        //
        (
            r"mt-?(\d+)",
            I32(|b, v| b.node.margin = UiRect::top(Val::Px(v as f32))),
        ),
        (
            r"mb-?(\d+)",
            I32(|b, v| b.node.margin = UiRect::bottom(Val::Px(v as f32))),
        ),
        (
            r"ml-?(\d+)",
            I32(|b, v| b.node.margin = UiRect::left(Val::Px(v as f32))),
        ),
        (
            r"mr-?(\d+)",
            I32(|b, v| b.node.margin = UiRect::right(Val::Px(v as f32))),
        ),
        (
            r"mx-?(\d+)",
            I32(|b, v| b.node.margin = UiRect::horizontal(Val::Px(v as f32))),
        ),
        (
            r"my-?(\d+)",
            I32(|b, v| b.node.margin = UiRect::vertical(Val::Px(v as f32))),
        ),
        (
            r"m-?(\d+)",
            I32(|b, v| b.node.margin = UiRect::all(Val::Px(v as f32))),
        ),
        //
        // Padding
        //
        (
            r"pt-?(\d+)",
            I32(|b, v| b.node.padding = UiRect::top(Val::Px(v as f32))),
        ),
        (
            r"pb-?(\d+)",
            I32(|b, v| b.node.padding = UiRect::bottom(Val::Px(v as f32))),
        ),
        (
            r"pl-?(\d+)",
            I32(|b, v| b.node.padding = UiRect::left(Val::Px(v as f32))),
        ),
        (
            r"pr-?(\d+)",
            I32(|b, v| b.node.padding = UiRect::right(Val::Px(v as f32))),
        ),
        (
            r"px-?(\d+)",
            I32(|b, v| b.node.padding = UiRect::horizontal(Val::Px(v as f32))),
        ),
        (
            r"py-?(\d+)",
            I32(|b, v| b.node.padding = UiRect::vertical(Val::Px(v as f32))),
        ),
        (
            r"p-?(\d+)",
            I32(|b, v| b.node.padding = UiRect::all(Val::Px(v as f32))),
        ),
        //
        // Color
        //
        (
            r"bg-rgb\(([\d\.]+),([\d\.]+),([\d\.]+)\)",
            F32F32F32(|bundle, r, g, b| {
                let color = Color::srgb(r, g, b);
                bundle.background_color = Some(BackgroundColor(color));
            }),
        ),
        (
            r"bg-rgba\(([\d\.]+),([\d\.]+),([\d\.]+),([\d\.]+)\)",
            F32F32F32F32(|bundle, r, g, b, a| {
                let color = Color::srgba(r, g, b, a);
                bundle.background_color = Some(BackgroundColor(color));
            }),
        ),
        (
            r"fg-white",
            Void(|b| {
                b.text_color = Some(TextColor(Color::WHITE));
            }),
        ),
        (
            r"fg-rgb\(([\d\.]+),([\d\.]+),([\d\.]+)\)",
            F32F32F32(|bundle, r, g, b| {
                let color = Color::srgb(r, g, b);
                bundle.text_color = Some(TextColor(color));
            }),
        ),
        (
            r"fg-rgba\(([\d\.]+),([\d\.]+),([\d\.]+),([\d\.]+)\)",
            F32F32F32F32(|bundle, r, g, b, a| {
                let color = Color::srgba(r, g, b, a);
                bundle.text_color = Some(TextColor(color));
            }),
        ),
        //
        // Outlines
        //
        (
            r"outline-width-([\d\.]+)",
            F32(|b, v| {
                b.outline.get_or_insert_with(Outline::default).width = Val::Px(v);
            }),
        ),
        (
            r"outline-rgb\(([\d\.]+),([\d\.]+),([\d\.]+)\)",
            F32F32F32(|bundle, r, g, b| {
                let color = Color::srgb(r, g, b);
                bundle.outline.get_or_insert_with(Outline::default).color = color;
            }),
        ),
        (
            r"outline-rgba\(([\d\.]+),([\d\.]+),([\d\.]+),([\d\.]+)\)",
            F32F32F32F32(|bundle, r, g, b, a| {
                let color = Color::srgba(r, g, b, a);
                bundle.outline.get_or_insert_with(Outline::default).color = color;
            }),
        ),
        //
        // Typography
        //
        (
            r"font-size-([\d.]+)",
            F32(|b, v| {
                b.text_font.get_or_insert_with(TextFont::default).font_size = v;
            }),
        ),
    ];

    let mut compiled = Vec::new();
    for (pattern, handler) in patterns {
        if let Ok(regex) = Regex::new(&format!("^{}$", pattern)) {
            compiled.push((regex, handler));
        } else {
            log::warn!("Invalid regex pattern in console styles: {}", pattern);
        }
    }

    compiled
});

/// Uses a tailwind-like shorthand to allow for more concise UI definitions
fn node_style(commands: &mut EntityCommands, sl: &str) {
    let mut bundle = StyledBundle::default();

    let tokens: Vec<&str> = sl.split_whitespace().collect();
    for token in tokens {
        let mut matched = false;

        for (regex, handler) in COMPILED_PATTERNS.iter() {
            use StyleHandler::*;
            let Some(captures) = regex.captures(token) else {
                continue;
            };
            matched = true;

            // Reminder first capture group is the whole match so all the length
            // checks are +1 of the number of arguments/sub-groups expected.
            match handler {
                Void(func) => {
                    if captures.len() != 1 {
                        log::warn!("Unexpected capture group for style: {}", token);
                        break;
                    }
                    func(&mut bundle);
                }
                I32(func) => {
                    if captures.len() != 2 {
                        log::warn!("No capture group for I32 style: {}", token);
                        break;
                    }

                    let Ok(value) = captures[1].parse::<i32>() else {
                        log::warn!("Invalid number in style: {}", token);
                        break;
                    };
                    func(&mut bundle, value);
                }
                F32(func) => {
                    if captures.len() != 2 {
                        log::warn!("No capture group for F32 style: {}", token);
                        break;
                    }
                    let Ok(value) = captures[1].parse::<f32>() else {
                        log::warn!("Invalid float in style: {}", token);
                        break;
                    };
                    func(&mut bundle, value);
                }
                Str(func) => {
                    if captures.len() != 2 {
                        log::warn!("No capture group for Str style: {}", token);
                        break;
                    }
                    func(&mut bundle, &captures[1]);
                }
                F32F32F32(func) => {
                    if captures.len() != 4 {
                        log::warn!(
                            "Incorrect number of capture groups for F32F32F32 style: {}",
                            token
                        );
                        break;
                    }
                    let Ok(v1) = captures[1].parse::<f32>() else {
                        log::warn!("Invalid first float in style: {}", token);
                        break;
                    };
                    let Ok(v2) = captures[2].parse::<f32>() else {
                        log::warn!("Invalid second float in style: {}", token);
                        break;
                    };
                    let Ok(v3) = captures[3].parse::<f32>() else {
                        log::warn!("Invalid third float in style: {}", token);
                        break;
                    };
                    func(&mut bundle, v1, v2, v3);
                }
                F32F32F32F32(func) => {
                    if captures.len() != 5 {
                        log::warn!(
                            "Incorrect number of capture groups for F32F32F32F32 style: {}",
                            token
                        );
                        break;
                    }
                    let Ok(v1) = captures[1].parse::<f32>() else {
                        log::warn!("Invalid first float in style: {}", token);
                        break;
                    };
                    let Ok(v2) = captures[2].parse::<f32>() else {
                        log::warn!("Invalid second float in style: {}", token);
                        break;
                    };
                    let Ok(v3) = captures[3].parse::<f32>() else {
                        log::warn!("Invalid third float in style: {}", token);
                        break;
                    };
                    let Ok(v4) = captures[4].parse::<f32>() else {
                        log::warn!("Invalid fourth float in style: {}", token);
                        break;
                    };
                    func(&mut bundle, v1, v2, v3, v4);
                }
            }
        }
        if !matched {
            log::warn!("Unknown style: {}", token);
        }
    }

    commands.insert(bundle.node);
    if let Some(z_index) = bundle.z_index {
        commands.insert(z_index);
    }
    if let Some(background_color) = bundle.background_color {
        commands.insert(background_color);
    }
    if let Some(text_font) = bundle.text_font {
        commands.insert(text_font);
    }
    if let Some(text_color) = bundle.text_color {
        commands.insert(text_color);
    }
    if let Some(outline) = bundle.outline {
        commands.insert(outline);
    }
}
