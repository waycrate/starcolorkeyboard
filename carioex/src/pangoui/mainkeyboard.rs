use cairo::Context;

use serde::{Deserialize, Serialize};

use std::sync::OnceLock;

use crate::KeyModifierType;

use super::contain_mode;

static MAIN_LAYOUT_INFO: OnceLock<Vec<Vec<MainLayout>>> = OnceLock::new();

const MAIN_LAYOUT: &str = include_str!("../../asserts/mainkeylayout/enUS.json");

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MainLayout {
    text: String,
    cap: Option<String>,
    shift: Option<String>,
    width: usize,
    line: usize,
    start_pos: usize,
    key: usize,
}

// TODO: cap and shift
#[allow(unused)]
#[derive(Clone, Copy)]
enum KeyType {
    Normal,
    Cap,
    Shift,
}

impl MainLayout {
    fn get_info(&self, keytype: KeyType, step: f64, font_size: i32) -> DrawInfo<'_> {
        match keytype {
            KeyType::Normal => DrawInfo {
                step,
                width: self.width as i32,
                font_size,
                line: self.line as i32,
                text: self.text.as_str(),
                start_pos: self.start_pos as i32,
            },
            KeyType::Cap => DrawInfo {
                step,
                width: self.width as i32,
                font_size,
                line: self.line as i32,
                text: match &self.cap {
                    Some(text) => text,
                    None => self.text.as_str(),
                },
                start_pos: self.start_pos as i32,
            },
            KeyType::Shift => DrawInfo {
                step,
                width: self.width as i32,
                font_size,
                line: self.line as i32,
                text: match &self.shift {
                    Some(text) => text,
                    None => match &self.cap {
                        Some(text) => text,
                        None => self.text.as_str(),
                    },
                },
                start_pos: self.start_pos as i32,
            },
        }
    }
}

fn get_keytype(key_type: KeyModifierType) -> KeyType {
    if contain_mode(key_type, KeyModifierType::Shift) {
        KeyType::Shift
    } else {
        KeyType::Normal
    }
}

fn get_main_layout() -> Vec<Vec<MainLayout>> {
    if let Some(layout_info) = MAIN_LAYOUT_INFO.get() {
        layout_info.clone()
    } else {
        let layout: Vec<Vec<MainLayout>> = serde_json::from_str(MAIN_LAYOUT).unwrap();
        MAIN_LAYOUT_INFO.set(layout.clone()).expect("Cannot set it");
        layout
    }
}

struct DrawInfo<'a> {
    step: f64,
    width: i32,
    font_size: i32,
    line: i32,
    text: &'a str,
    start_pos: i32,
}

fn draw_unit_key(
    pangolayout: &pango::Layout,
    content: &Context,
    DrawInfo {
        step,
        width,
        font_size,
        line,
        text,
        start_pos,
    }: DrawInfo,
) {
    let start_x = step * start_pos as f64 / 2.0;
    let end_x = step * width as f64 / 2.0 + start_x;
    let start_y = step * line as f64;
    let end_y = step * (line + 1) as f64;
    content.move_to(start_x, start_y);
    content.line_to(start_x, end_y);
    content.move_to(end_x, start_y);
    content.line_to(end_x, end_y);

    content.move_to(start_x, start_y);
    content.line_to(end_x, start_y);
    content.move_to(start_x, end_y);
    content.line_to(end_x, end_y);

    content.stroke().unwrap();

    pangolayout.set_text(text);
    let font_adjusty = step / 2.0 - font_size as f64;
    content.save().unwrap();
    content.move_to(start_x + font_adjusty, start_y);
    pangocairo::show_layout(content, pangolayout);
    content.restore().unwrap();
}

pub(crate) fn draw_main_keyboard(
    content: &Context,
    pangolayout: &pango::Layout,
    height: i32,
    font_size: i32,
    key_type: KeyModifierType,
) {
    let step = height / 4;

    let keytype = get_keytype(key_type);
    for oneline in get_main_layout().iter() {
        for map in oneline.iter() {
            draw_unit_key(
                pangolayout,
                content,
                map.get_info(keytype, step as f64, font_size),
            );
        }
    }
}

pub fn find_keycode_from_mainkeyboard((pos_x, pos_y): (i32, i32), step: i32) -> Option<u32> {
    let main_layout = get_main_layout();
    let aby = pos_y / step;
    if aby >= main_layout.len() as i32 {
        return None;
    }
    for map in main_layout[aby as usize].iter() {
        let MainLayout {
            width,
            start_pos,
            key,
            ..
        } = map;
        if pos_x > *start_pos as i32 * step / 2
            && pos_x < (*start_pos as i32 + *width as i32) * step / 2
        {
            return Some(*key as u32);
        }
    }
    None
}
