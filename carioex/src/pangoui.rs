//use std::f64::consts::PI;

use cairo::{self, Context};

//const RADIUS: f64 = 150_f64;
//const N_WORDS: i32 = 10;
fn draw_number_keyboard(content: &Context, width: i32, height: i32, font_size: i32) {
    // NOTE: here require width > height
    assert!(width > height);

    let step = height as f64 / 3.0;
    let x_1 = width as f64 - 4.0 * step;
    let x_2 = width as f64 - 3.0 * step;
    let x_3 = width as f64 - 2.0 * step;
    let x_4 = width as f64 - step;
    let x_5 = width as f64;

    let y_1 = 0.0;
    let y_2 = step;
    let y_3 = 2.0 * step;
    let y_4 = height as f64;

    let font_adjusty = step / 2.0 - font_size as f64;
    let font_adjustx = step / 2.0 - font_size as f64 / 2.0;
    content.set_source_rgb(0.0, 0.0, 0.0);
    content.move_to(x_1, y_1);
    content.line_to(x_1, y_4);
    content.move_to(x_2, y_1);
    content.line_to(x_2, y_4);
    content.move_to(x_3, y_1);
    content.line_to(x_3, y_4);
    content.move_to(x_4, y_1);
    content.line_to(x_4, y_4);
    content.move_to(x_5, y_1);
    content.line_to(x_5, y_4);

    content.move_to(x_1, y_1);
    content.line_to(x_5, y_1);
    content.move_to(x_1, y_2);
    content.line_to(x_5, y_2);
    content.move_to(x_1, y_3);
    content.line_to(x_5, y_3);
    content.move_to(x_1, y_4);
    content.line_to(x_5, y_4);

    content.stroke().unwrap();

    let pangolayout = pangocairo::create_layout(content);
    let mut desc = pango::FontDescription::new();
    desc.set_family("Sans");
    desc.set_weight(pango::Weight::Bold);
    desc.set_size(font_size * pango::SCALE);
    pangolayout.set_font_description(Some(&desc));

    pangolayout.set_text("1");
    content.save().unwrap();
    content.move_to(x_1 + font_adjustx, y_1 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("2");
    content.save().unwrap();
    content.move_to(x_2 + font_adjustx, y_1 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("3");
    content.save().unwrap();
    content.move_to(x_3 + font_adjustx, y_1 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("4");
    content.save().unwrap();
    content.move_to(x_1 + font_adjustx, y_2 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("5");
    content.save().unwrap();
    content.move_to(x_2 + font_adjustx, y_2 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("6");
    content.save().unwrap();
    content.move_to(x_3 + font_adjustx, y_2 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("7");
    content.save().unwrap();
    content.move_to(x_1 + font_adjustx, y_3 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("8");
    content.save().unwrap();
    content.move_to(x_2 + font_adjustx, y_3 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("9");
    content.save().unwrap();
    content.move_to(x_3 + font_adjustx, y_3 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();

    pangolayout.set_text("0");
    content.save().unwrap();
    content.move_to(x_4 + font_adjustx, y_2 + font_adjusty);
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();
}

#[derive(Debug, Default)]
pub struct PangoUi {
    width: i32,
    height: i32,
}

impl PangoUi {
    pub fn ui(&self) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let height = self.height;
        let width = self.width;
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        let cr = cairo::Context::new(&surface).unwrap();
        cr.set_source_rgb(1_f64, 1_f64, 1_f64);
        cr.paint().unwrap();

        draw_number_keyboard(&cr, width, height, 27);

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
            return None;
        } else if pos_x > x_4 {
            if pos_y / step == 1 {
                return Some(11);
            } else {
                return None;
            }
        }
        let abx = (pos_x - x_1) / step;
        let aby = pos_y / step;
        let code = aby * 3 + abx + 2;
        Some(code as u32)
    }
}
//pub fn ui(width: i32, height: i32) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
//    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
//    let cr = cairo::Context::new(&surface).unwrap();
//    cr.set_source_rgb(1_f64, 1_f64, 1_f64);
//    cr.paint().unwrap();
//
//    draw_number_keyboard(&cr, width, height, 27);
//
//    use std::io::Cursor;
//    let mut buff = Cursor::new(Vec::new());
//
//    surface.write_to_png(&mut buff).unwrap();
//    image::load_from_memory_with_format(buff.get_ref(), image::ImageFormat::Png)
//        .unwrap()
//        .to_rgba8()
//}
