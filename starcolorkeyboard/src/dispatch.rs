use crate::otherkeys;

use super::State;

use wayland_client::{
    protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_registry,
        wl_seat, wl_shm, wl_shm_pool, wl_surface, wl_touch,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};

use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use wayland_protocols::xdg::shell::client::xdg_wm_base;

use wayland_protocols::xdg::xdg_output::zv1::client::{
    zxdg_output_manager_v1::{self, ZxdgOutputManagerV1},
    zxdg_output_v1::{self, ZxdgOutputV1},
};

use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};

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
                //println!("{wl_output:?}");
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
                state.wl_composer = Some(compositor);
            } else if interface == wl_shm::WlShm::interface().name {
                state.wl_shm = Some(registry.bind::<wl_shm::WlShm, _, _>(name, version, qh, ()));
            } else if interface == wl_seat::WlSeat::interface().name {
                state.wl_seat = Some(registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ()));
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
            } else if interface == ZwpVirtualKeyboardManagerV1::interface().name {
                let virtual_keyboard_manager =
                    registry.bind::<ZwpVirtualKeyboardManagerV1, _, _>(name, version, qh, ());
                state.virtual_keyboard_manager = Some(virtual_keyboard_manager);
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        _event: <wl_output::WlOutput as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        //if let wl_output::Event::Mode { width, height, .. } = _event {
        //    //state.wl_size.push((width, height));
        //    println!("{width}, {height}");
        //}
    }
}

impl Dispatch<ZxdgOutputV1, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &ZxdgOutputV1,
        event: <ZxdgOutputV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let zxdg_output_v1::Event::LogicalSize { width, height } = event {
            if state.zwl_size.len() != state.zxdg_output.len() {
                state.zwl_size.push((width, height));
                return;
            }
            if let Some(index) = state
                .zxdg_output
                .iter()
                .position(|zoutput| zoutput == proxy)
            {
                state.zwl_size[index] = (width, height);
                // TODO: if is the layer
                if index == 0 {
                    state.keyboard_ui.as_mut().unwrap().set_size((width, 300));
                    state.update_map(qh);
                }
            }
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
        _state: &mut Self,
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
            if capabilities.contains(wl_seat::Capability::Pointer) {
                seat.get_pointer(qh, ());
            }
            if capabilities.contains(wl_seat::Capability::Touch) {
                seat.get_touch(qh, ());
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

impl Dispatch<wl_callback::WlCallback, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_callback::WlCallback,
        _event: <wl_callback::WlCallback as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for State {
    fn event(
        wlstate: &mut Self,
        _proxy: &wl_pointer::WlPointer,
        event: <wl_pointer::WlPointer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Button { state, .. } => match state {
                WEnum::Value(wl_pointer::ButtonState::Pressed) => {
                    if let Some(key) = wlstate.get_key_point() {
                        if !otherkeys::is_unique_key(key) {
                            wlstate.key_press(key);
                        }
                    }
                }
                WEnum::Value(wl_pointer::ButtonState::Released) => {
                    if let Some(key) = wlstate.get_key_point() {
                        if otherkeys::is_unique_key(key) {
                            if key == otherkeys::CLOSE_KEYBOARD {
                                wlstate.running = false;
                            } else if key == otherkeys::MIN_KEYBOARD {
                                wlstate.keyboard_ui.as_mut().unwrap().set_min();
                                wlstate.min_keyboard();
                            }
                            return;
                        }
                        if wlstate.key_release(key) {
                            wlstate.update_map(qh);
                        }
                    }
                }
                _ => {}
            },
            wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            } => {
                wlstate
                    .keyboard_ui
                    .as_mut()
                    .unwrap()
                    .set_point_pos((surface_x, surface_y));
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                wlstate
                    .keyboard_ui
                    .as_mut()
                    .unwrap()
                    .set_point_pos((surface_x, surface_y));
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_touch::WlTouch, ()> for State {
    fn event(
        wlstate: &mut Self,
        _proxy: &wl_touch::WlTouch,
        event: <wl_touch::WlTouch as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_touch::Event::Down { x, y, .. } => {
                wlstate.keyboard_ui.as_mut().unwrap().set_touch_pos((x, y));
                if let Some(key) = wlstate.get_key_touch() {
                    wlstate.key_press(key);
                }
            }
            wl_touch::Event::Up { .. } => {
                if let Some(key) = wlstate.get_key_touch() {
                    wlstate.key_press(key);
                }
            }
            _ => {}
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
        // TODO: if is the same surface
        if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
            surface.ack_configure(serial);
            let keyboardui = state.keyboard_ui.as_ref().unwrap();
            if keyboardui.is_same_surface(surface) {
                keyboardui.surface_refresh();
            }
        }
    }
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpVirtualKeyboardManagerV1,
        _event: <ZwpVirtualKeyboardManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardV1, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpVirtualKeyboardV1,
        _event: <ZwpVirtualKeyboardV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}
