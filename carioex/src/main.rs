mod dispatch;
mod pangoui;
use std::{fs::File, os::unix::prelude::AsRawFd};
use wayland_client::{
    protocol::{wl_buffer, wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::{self, Anchor},
};

use wayland_protocols::xdg::shell::client::xdg_wm_base;

use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1;

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let mut event_queue = conn.new_event_queue();
    let qhandle = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qhandle, ());

    let mut state = State {
        running: true,
        wl_output: vec![],
        wl_size: vec![],
        wl_shm: None,
        base_surface: None,
        layer_shell: None,
        layer_surface: None,
        buffer: None,
        wm_base: None,
        xdg_output_manager: None,
    };

    event_queue.blocking_dispatch(&mut state).unwrap();
    let mut displays: usize = 0;
    while displays < state.wl_output.len() + 1 {
        event_queue.blocking_dispatch(&mut state).unwrap();
        displays = state.wl_output.len() + 1;
    }
    for index in 0..state.wl_output.len() {
        state.xdg_output_manager.as_ref().unwrap().get_xdg_output(
            &state.wl_output[index],
            &qhandle,
            (),
        );
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
    if state.layer_shell.is_some() && state.wm_base.is_some() {
        state.set_buffer(state.get_size_from_display(0), &qhandle);
        state.init_layer_surface(
            &qhandle,
            state.get_size_from_display(0),
            Some(&state.wl_output[0].clone()),
        );
    }

    while state.running {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

struct State {
    running: bool,
    wl_output: Vec<wl_output::WlOutput>,
    wl_size: Vec<(i32, i32)>,
    wl_shm: Option<wl_shm::WlShm>,
    base_surface: Option<wl_surface::WlSurface>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    buffer: Option<wl_buffer::WlBuffer>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg_output_manager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
}

impl State {
    fn set_buffer(&mut self, (width, height): (i32, i32), qh: &QueueHandle<Self>) {
        let shm = self.wl_shm.as_ref().unwrap();
        let mut file = tempfile::tempfile().unwrap();
        draw(&mut file, (width, height));
        let pool = shm.create_pool(file.as_raw_fd(), width * height * 4, qh, ());
        let buffer = pool.create_buffer(
            0,
            width,
            height,
            width * 4,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );
        self.buffer = Some(buffer);
    }

    fn get_size_from_display(&self, index: usize) -> (i32, i32) {
        (self.wl_size[index].0, 300)
    }
}

impl State {
    fn init_layer_surface(
        &mut self,
        qh: &QueueHandle<State>,
        (_width, height): (i32, i32),
        output: Option<&wl_output::WlOutput>,
    ) {
        let layer = self.layer_shell.as_ref().unwrap().get_layer_surface(
            self.base_surface.as_ref().unwrap(),
            output,
            Layer::Top,
            "precure".to_string(),
            qh,
            (),
        );
        layer.set_anchor(Anchor::Bottom | Anchor::Right | Anchor::Left);
        layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);
        layer.set_exclusive_zone(height);
        layer.set_size(0, height as u32);
        self.base_surface.as_ref().unwrap().commit();

        self.layer_surface = Some(layer);
    }
}

fn draw(tmp: &mut File, (_buf_x, _buf_y): (i32, i32)) {
    use std::io::Write;

    let mut buf = std::io::BufWriter::new(tmp);

    for index in pangoui::ui(_buf_x, _buf_y).pixels() {
        buf.write_all(&index.0).unwrap();
    }
    buf.flush().unwrap();
}
