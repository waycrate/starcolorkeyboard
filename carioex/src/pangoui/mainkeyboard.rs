use cairo::Context;

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
    let start_x = step * start_pos as f64;
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
            start_pos: 0,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "q",
            start_pos: 2,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "w",
            start_pos: 3,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "e",
            start_pos: 4,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "r",
            start_pos: 5,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "t",
            start_pos: 6,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "y",
            start_pos: 7,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "u",
            start_pos: 8,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "i",
            start_pos: 9,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "o",
            start_pos: 10,
        },
    );
    draw_unit_key(
        pangolayout,
        content,
        DrawInfo {
            step: step as f64,
            width: 1,
            font_size,
            line: 0,
            text: "p",
            start_pos: 11,
        },
    );
}
