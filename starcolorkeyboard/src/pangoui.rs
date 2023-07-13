mod mainkeyboard;
mod smallkeyboard;
//use std::f64::consts::PI;

use smallkeyboard::{draw_number_keyboard, find_keycode_from_smallkeyboard};

use crate::{otherkeys, pangoui::mainkeyboard::draw_main_keyboard};

use self::mainkeyboard::find_keycode_from_mainkeyboard;

use super::KeyModifierType;

use crate::consts::{EXCULDE_ZONE_RIGHT, EXCULDE_ZONE_TOP};

#[derive(Debug, Default)]
pub struct PangoUi {
    width: i32,
    height: i32,
}

fn contain_mode(key_type: KeyModifierType, mode: KeyModifierType) -> bool {
    key_type == key_type | mode
}

impl PangoUi {
    pub(crate) fn ui(
        &self,
        key_type: KeyModifierType,
    ) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let height = self.height;
        let width = self.width;
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let cr = cairo::Context::new(&surface).unwrap();
        cr.set_source_rgb(1_f64, 1_f64, 1_f64);
        cr.paint().unwrap();
        let font_size = 23;
        let pangolayout = pangocairo::create_layout(&cr);
        let mut desc = pango::FontDescription::new();
        desc.set_family("Sans");
        desc.set_weight(pango::Weight::Bold);

        desc.set_size(font_size * pango::SCALE);
        pangolayout.set_font_description(Some(&desc));

        draw_number_keyboard(&cr, &pangolayout, width, height, 27, key_type);
        draw_main_keyboard(&cr, &pangolayout, height, 27, key_type);

        use std::io::Cursor;
        let mut buff = Cursor::new(Vec::new());

        surface.write_to_png(&mut buff).unwrap();
        image::load_from_memory_with_format(buff.get_ref(), image::ImageFormat::Png)
            .unwrap()
            .to_rgba8()
    }

    pub fn set_size(&mut self, (width, height): (i32, i32)) {
        self.width = width;
        self.height = height;
    }

    pub fn get_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub fn get_key(&self, (pos_x, pos_y): (f64, f64)) -> Option<u32> {
        let (pos_x, pos_y) = (pos_x as i32, pos_y as i32);
        let exclude_zone = EXCULDE_ZONE_TOP as i32;
        let step = (self.height - exclude_zone) / 3;
        let x_1 = self.width - 4 * step;
        let x_4 = self.width - step;
        let x_exclude = self.width - EXCULDE_ZONE_RIGHT as i32;
        if pos_y < EXCULDE_ZONE_TOP as i32 {
            if pos_x < x_exclude {
                return None;
            }
            let step_right = EXCULDE_ZONE_TOP as i32;
            let right_w = pos_x - x_exclude;
            if right_w / step_right == 0 {
                return Some(otherkeys::MIN_KEYBOARD);
            } else {
                return Some(otherkeys::CLOSE_KEYBOARD);
            }
        }
        if pos_x < x_1 {
            let step = (self.height - exclude_zone) / 4;
            return find_keycode_from_mainkeyboard((pos_x, pos_y), step);
        } else if pos_x > x_4 {
            match (pos_y - exclude_zone) / step {
                0 => return Some(12),
                1 => return Some(11),
                2 => return Some(13),
                _ => return None,
            }
        }
        Some(find_keycode_from_smallkeyboard((pos_x, pos_y), x_1, step))
    }
}
