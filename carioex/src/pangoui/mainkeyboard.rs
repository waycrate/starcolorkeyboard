use cairo::Context;

fn draw_unit_key(
    pangolayout: &pango::Layout,
    content: &Context,
    step: f64,
    width: i32,
    font_size: i32,
    line: i32,
    text: &str,
    start_x: f64,
) {
    let end_x = step * width as f64 + start_x;
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
    pangocairo::show_layout(content, &pangolayout);
    content.restore().unwrap();
}

pub fn draw_main_keyboard(content: &Context, height: i32, font_size: i32) {
    let step = height / 4;
    let pangolayout = pangocairo::create_layout(content);
    let mut desc = pango::FontDescription::new();
    desc.set_family("Sans");
    desc.set_weight(pango::Weight::Bold);
    desc.set_size(font_size * pango::SCALE);
    pangolayout.set_font_description(Some(&desc));
    draw_unit_key(
        &pangolayout,
        content,
        step as f64,
        2,
        font_size,
        0,
        "Tab",
        0.0,
    );
}
