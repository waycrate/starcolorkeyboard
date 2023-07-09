use cairo::Context;

struct DrawInfo<'a> {
    step: f64,
    width: i32,
    font_size: i32,
    line: i32,
    text: &'a str,
    start_x: f64,
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
        start_x,
    }: DrawInfo,
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
    pangocairo::show_layout(content, pangolayout);
    content.restore().unwrap();
}

pub fn draw_main_keyboard(
    content: &Context,
    pangolayout: &pango::Layout,
    height: i32,
    font_size: i32,
) {
    let step = height / 4;
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 2,
            font_size,
            line: 0,
            text: "Tab",
            start_x: 0.0,
        },
    );
}
