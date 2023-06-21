mod pangoui;
use std::{fs::File, os::unix::prelude::AsRawFd};
use wayland_client::{
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_output, wl_registry, wl_seat, wl_shm,
        wl_shm_pool, wl_surface,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::{self, Anchor},
};

use wayland_protocols::xdg::shell::client::xdg_wm_base;

use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_manager_v1::{
    self, ZxdgOutputManagerV1,
};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1::{self, ZxdgOutputV1};

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
        self.wl_size[index]
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == wl_output::WlOutput::interface().name {
                //&& state.wl_output.is_none() {
                let wl_output = registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
                println!("{wl_output:?}");
                state.wl_output.push(wl_output);
            } else if interface == zwlr_layer_shell_v1::ZwlrLayerShellV1::interface().name {
                let wl_layer = registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                    name,
                    version,
                    qh,
                    (),
                );
                state.layer_shell = Some(wl_layer);
            } else if interface == wl_compositor::WlCompositor::interface().name {
                let compositor =
                    registry.bind::<wl_compositor::WlCompositor, _, _>(name, version, qh, ());
                let surface = compositor.create_surface(qh, ());
                state.base_surface = Some(surface);
            } else if interface == wl_shm::WlShm::interface().name {
                state.wl_shm = Some(registry.bind::<wl_shm::WlShm, _, _>(name, version, qh, ()));
            } else if interface == wl_seat::WlSeat::interface().name {
                registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());
            } else if interface == xdg_wm_base::XdgWmBase::interface().name {
                let wm_base = registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, 1, qh, ());
                state.wm_base = Some(wm_base);
            } else if interface == zxdg_output_manager_v1::ZxdgOutputManagerV1::interface().name {
                let xdg_output_manager = registry
                    .bind::<zxdg_output_manager_v1::ZxdgOutputManagerV1, _, _>(
                        name,
                        version,
                        qh,
                        (),
                    );
                state.xdg_output_manager = Some(xdg_output_manager);
            }
        }
    }
}
impl Dispatch<wl_output::WlOutput, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: <wl_output::WlOutput as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Mode { width, height, .. } = event {
            //state.wl_size.push((width, height));
            println!("{width}, {height}");
        }
    }
}
impl Dispatch<ZxdgOutputV1, ()> for State {
    fn event(
        state: &mut Self,
        _proxy: &ZxdgOutputV1,
        event: <ZxdgOutputV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let zxdg_output_v1::Event::LogicalSize { width, height } = event {
            state.wl_size.push((width, height));
        }
    }
}

impl Dispatch<ZxdgOutputManagerV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZxdgOutputManagerV1,
        _event: <ZxdgOutputManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_compositor::WlCompositor, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // wl_compositor has no event
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // we ignore wl_surface events in this example
    }
}

impl Dispatch<wl_shm::WlShm, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // we ignore wl_shm events in this example
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // we ignore wl_shm_pool events in this example
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // we ignore wl_buffer events in this example
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

impl State {
    fn init_layer_surface(
        &mut self,
        qh: &QueueHandle<State>,
        (width, height): (i32, i32),
        output: Option<&wl_output::WlOutput>,
    ) {
        let layer = self.layer_shell.as_ref().unwrap().get_layer_surface(
            self.base_surface.as_ref().unwrap(),
            output,
            Layer::Overlay,
            "precure".to_string(),
            qh,
            (),
        );
        layer.set_anchor(Anchor::Bottom | Anchor::Right);
        layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand);
        layer.set_size(width as u32, height as u32);
        self.base_surface.as_ref().unwrap().commit();

        self.layer_surface = Some(layer);
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            println!("key it is {key}");
            if key == 1 {
                // ESC key
                state.running = false;
            }
        }
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: <zwlr_layer_shell_v1::ZwlrLayerShellV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for State {
    fn event(
        state: &mut Self,
        surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: <zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
            surface.ack_configure(serial);
            let surface = state.base_surface.as_ref().unwrap();
            if let Some(ref buffer) = state.buffer {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}
