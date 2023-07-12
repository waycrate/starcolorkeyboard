mod mainkeyboard;
mod smallkeyboard;
//use std::f64::consts::PI;

use smallkeyboard::{draw_number_keyboard, find_keycode_from_smallkeyboard};

use crate::pangoui::mainkeyboard::draw_main_keyboard;

use self::mainkeyboard::find_keycode_from_mainkeyboard;

use bitflags::bitflags;

#[derive(Debug, Default)]
pub struct PangoUi {
    width: i32,
    height: i32,
}

bitflags! {
    #[allow(unused)]
    #[derive(Debug)]
    struct KeyModifierType : u32 {
        const NoMod = 0;
        const Shift = 1;
        const CapsLock = 2;
        const Ctrl = 4;
        const Alt = 8;
        const Super = 64;
        const AltGr = 128;
    }
}

impl PangoUi {
    pub fn ui(&self) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let height = self.height;
        let width = self.width;
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let cr = cairo::Context::new(&surface).unwrap();
        cr.set_source_rgb(1_f64, 1_f64, 1_f64);
        cr.paint().unwrap();
        let font_size = 27;
        let pangolayout = pangocairo::create_layout(&cr);
        let mut desc = pango::FontDescription::new();
        desc.set_family("Sans");
        desc.set_weight(pango::Weight::Bold);
        desc.set_size(font_size * pango::SCALE);
        pangolayout.set_font_description(Some(&desc));

        draw_number_keyboard(&cr, &pangolayout, width, height, 27);
        draw_main_keyboard(&cr, &pangolayout, height, 27);

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
        let step = self.height / 3;
        let x_1 = self.width - 4 * step;
        let x_4 = self.width - step;

        if pos_x < x_1 {
            let step = self.height / 4;
            return find_keycode_from_mainkeyboard((pos_x, pos_y), step);
        } else if pos_x > x_4 {
            if pos_y / step == 1 {
                return Some(11);
            } else {
                return None;
            }
        }
        Some(find_keycode_from_smallkeyboard((pos_x, pos_y), x_1, step))
    }
}
