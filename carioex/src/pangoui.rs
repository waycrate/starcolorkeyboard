use std::f64::consts::PI;

use cairo::{self, Context};

const RADIUS: f64 = 150_f64;
const N_WORDS: i32 = 10;

fn draw_text(content: &Context) {
    let pango_scale = pango::SCALE;
    content.translate(RADIUS, RADIUS);
    let pangolayout = pangocairo::create_layout(content);
    let mut desc = pango::FontDescription::new();
    pangolayout.set_text("Text");

    desc.set_family("Sans");
    desc.set_weight(pango::Weight::Bold);
    desc.set_size(27 * pango_scale);
    pangolayout.set_font_description(Some(&desc));

    for i in 0..N_WORDS {
        let angle: f64 = (360 * i) as f64 / N_WORDS as f64;

        content.save().unwrap();
        let red: f64 = (1_f64 + f64::cos((angle - 60.0) * PI / 180_f64)) / 2_f64;

        content.set_source_rgb(red, 0.0, 1.0 - red);
        content.rotate(angle * PI / 180.0);

        pangocairo::update_layout(content, &pangolayout);

        let (width, _height) = pangolayout.size();

        content.move_to(
            (width as f64 / pango_scale as f64) * -1.0 / 2.0,
            -1.0 * RADIUS,
        );
        pangocairo::show_layout(content, &pangolayout);
        content.restore().unwrap();
    }
}

pub fn ui() -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let surface =
        cairo::ImageSurface::create(cairo::Format::ARgb32, 2 * RADIUS as i32, 2 * RADIUS as i32)
            .unwrap();
    let cr = cairo::Context::new(&surface).unwrap();
    cr.set_source_rgb(1_f64, 1_f64, 1_f64);
    cr.paint().unwrap();

    draw_text(&cr);

    use std::io::Cursor;
    let mut buff = Cursor::new(Vec::new());

    surface.write_to_png(&mut buff).unwrap();
    image::load_from_memory_with_format(buff.get_ref(), image::ImageFormat::Png)
        .unwrap()
        .to_rgba8()
}
