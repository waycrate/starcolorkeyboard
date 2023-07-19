mod consts;
mod dispatch;
mod keyboardlayouts;
#[allow(unused)]
mod otherkeys;
mod pangoui;
use std::{ffi::CString, fs::File, io::Write, os::unix::prelude::AsRawFd, path::PathBuf};

use consts::EXCULDE_ZONE_TOP;
use keyboardlayouts::Layouts;

use wayland_client::{
    protocol::{
        wl_buffer, wl_compositor,
        wl_keyboard::{self, KeyState},
        wl_output, wl_seat, wl_shm, wl_surface,
    },
    Connection, QueueHandle,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, Layer},
    zwlr_layer_surface_v1::{self, Anchor},
};

use wayland_protocols::xdg::shell::client::xdg_wm_base;

use wayland_protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1};

use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};

use xkbcommon::xkb;

use pangoui::PangoUi;

use bitflags::bitflags;

bitflags! {
    #[allow(unused)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl From<u32> for KeyModifierType {
    fn from(value: u32) -> Self {
        match value {
            otherkeys::CAPS_LOCK => KeyModifierType::CapsLock,
            otherkeys::SHIFT_LEFT | otherkeys::SHIFT_RIGHT => KeyModifierType::Shift,
            otherkeys::MENU => KeyModifierType::Super,
            otherkeys::CTRL_LEFT | otherkeys::CTRL_RIGHT => KeyModifierType::Ctrl,
            otherkeys::ALT_LEFT | otherkeys::ALT_RIGHT => KeyModifierType::Alt,
            _ => KeyModifierType::NoMod,
        }
    }
}

impl From<usize> for KeyModifierType {
    fn from(value: usize) -> Self {
        let value = value as u32;
        value.into()
    }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let mut event_queue = conn.new_event_queue();
    let qhandle = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qhandle, ());

    let mut state = State::init();

    event_queue.blocking_dispatch(&mut state).unwrap();
    let mut displays: usize = 0;
    while displays < state.wl_output.len() + 1 {
        event_queue.blocking_dispatch(&mut state).unwrap();
        displays = state.wl_output.len() + 1;
    }
    for index in 0..state.wl_output.len() {
        let zxdg_output = state.xdg_output_manager.as_ref().unwrap().get_xdg_output(
            &state.wl_output[index],
            &qhandle,
            (),
        );
        state.zxdg_output.push(zxdg_output);
    }
    event_queue.blocking_dispatch(&mut state).unwrap();

    if state.layer_shell.is_some() && state.wm_base.is_some() {
        state.init_virtual_keyboard(&qhandle);
        let mut pangoui = pangoui::PangoUi::default();
        pangoui.set_size(state.get_size_from_display(0));
        //state.pangoui.set_size(state.get_size_from_display(0));
        let buffer = state.init_buffer(
            &qhandle,
            KeyModifierType::NoMod,
            &pangoui,
            state.get_size_from_display(0),
        );
        let (base_surface, layer_surface) = state.init_layer_surface(
            &qhandle,
            state.get_size_from_display(0),
            Some(&state.wl_output[0].clone()),
        );
        state.keyboard_ui = Some(KeyboardSurface {
            base_surface,
            layer_surface,
            pangoui,
            buffer,
            position: (0.0, 0.0),
            touch_pos: (0.0, 0.0),
            is_min: false,
        })
    }

    while state.running {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

#[allow(unused)]
#[derive(Debug)]
struct KeyboardSurface {
    base_surface: wl_surface::WlSurface,
    layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pangoui: PangoUi,
    buffer: wl_buffer::WlBuffer,
    position: (f64, f64),
    touch_pos: (f64, f64),
    is_min: bool,
}

#[allow(unused)]
impl KeyboardSurface {
    fn set_min(&mut self) {
        self.is_min = !self.is_min
    }
    fn set_touch_pos(&mut self, (x, y): (f64, f64)) {
        self.touch_pos = (x, y)
    }
    fn set_point_pos(&mut self, (x, y): (f64, f64)) {
        self.position = (x, y)
    }
    fn draw(&mut self, key_type: KeyModifierType, tmp: &File) {
        let mut buf = std::io::BufWriter::new(tmp);

        for index in self.pangoui.ui(key_type).pixels() {
            buf.write_all(&index.0).unwrap();
        }
        buf.flush().unwrap();
    }

    fn set_size(&mut self, (width, height): (i32, i32)) {
        self.pangoui.set_size((width, height));
    }

    fn get_size(&self) -> (i32, i32) {
        self.pangoui.get_size()
    }

    fn surface_refresh(&self) {
        self.base_surface.attach(Some(&self.buffer), 0, 0);
        self.base_surface.commit();
    }

    fn get_key_point(&self) -> Option<u32> {
        self.pangoui.get_key(self.position)
    }

    fn get_key_touch(&self) -> Option<u32> {
        self.pangoui.get_key(self.touch_pos)
    }

    fn ui(&self, key_type: KeyModifierType) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        self.pangoui.ui(key_type)
    }

    fn min_keyboard(&self) {
        let layer_surf = &self.layer_surface;
        if self.is_min {
            layer_surf.set_size(0, EXCULDE_ZONE_TOP as u32);
            layer_surf.set_exclusive_zone(EXCULDE_ZONE_TOP as i32);
        } else {
            let (_, height) = self.pangoui.get_size();
            layer_surf.set_size(0, height as u32);
            layer_surf.set_exclusive_zone(height);
        }

        self.base_surface.commit();
    }

    fn get_base_surface(&self) -> &wl_surface::WlSurface {
        &self.base_surface
    }

    fn update_map(&mut self, buffer: wl_buffer::WlBuffer, qh: &QueueHandle<State>) {
        self.buffer = buffer;
        let (width, height) = self.get_size();
        self.base_surface.damage_buffer(0, 0, width, height);
        self.base_surface.frame(qh, ());
        self.base_surface.attach(Some(&self.buffer), 0, 0);
        self.base_surface.commit();
    }
}

struct State {
    // running state
    running: bool,

    // to init zxdg_output
    wl_output: Vec<wl_output::WlOutput>,

    // base shell
    wl_shm: Option<wl_shm::WlShm>,
    wl_seat: Option<wl_seat::WlSeat>,
    wl_composer: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,

    // keyboard ui
    keyboard_ui: Option<KeyboardSurface>,

    // size and output
    zxdg_output: Vec<zxdg_output_v1::ZxdgOutputV1>,
    zwl_size: Vec<(i32, i32)>,
    xdg_output_manager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,

    // keyboard
    virtual_keyboard_manager: Option<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1>,
    virtual_keyboard: Option<zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1>,
    xkb_state: xkb::State,
    keymode: KeyModifierType,
}

impl State {
    fn init() -> Self {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);

        let keymap = xkb::Keymap::new_from_names(
            &context,
            "",
            "",
            Layouts::EnglishUs.to_layout_name(), // if no , it is norwegian
            "",
            None,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .expect("xkbcommon keymap panicked!");
        State {
            running: true,
            wl_output: vec![],
            zwl_size: vec![],
            wl_shm: None,
            wl_seat: None,
            wl_composer: None,
            layer_shell: None,
            keyboard_ui: None,
            wm_base: None,
            xdg_output_manager: None,
            zxdg_output: vec![],
            virtual_keyboard_manager: None,
            virtual_keyboard: None,
            xkb_state: xkb::State::new(&keymap),
            keymode: KeyModifierType::NoMod,
        }
    }
    fn init_buffer(
        &mut self,
        qh: &QueueHandle<Self>,
        key_type: KeyModifierType,
        pangoui: &PangoUi,
        (width, height): (i32, i32),
    ) -> wl_buffer::WlBuffer {
        //let (width, height) = self.pangoui.get_size();
        let file = tempfile::tempfile().unwrap();
        self.init_draw(key_type, pangoui, &file);
        let shm = self.wl_shm.as_ref().unwrap();
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
        buffer
    }
    // TODO: move to two funciton
    fn set_buffer(
        &mut self,
        qh: &QueueHandle<Self>,
        key_type: KeyModifierType,
        (width, height): (i32, i32),
    ) -> wl_buffer::WlBuffer {
        //let (width, height) = self.pangoui.get_size();
        let file = tempfile::tempfile().unwrap();
        self.draw(key_type, &file);
        let shm = self.wl_shm.as_ref().unwrap();
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
        buffer
    }

    // TODO : change it
    fn min_keyboard(&self) {
        self.keyboard_ui.as_ref().unwrap().min_keyboard();
    }

    fn get_size_from_display(&self, index: usize) -> (i32, i32) {
        (self.zwl_size[index].0, 300)
    }

    // TODO: change it
    fn init_layer_surface(
        &mut self,
        qh: &QueueHandle<State>,
        (_width, height): (i32, i32),
        output: Option<&wl_output::WlOutput>,
    ) -> (
        wl_surface::WlSurface,
        zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    ) {
        let surface = self.wl_composer.as_ref().unwrap().create_surface(qh, ());
        let layer = self.layer_shell.as_ref().unwrap().get_layer_surface(
            &surface,
            output,
            Layer::Overlay,
            "precure".to_string(),
            qh,
            (),
        );
        layer.set_anchor(Anchor::Bottom | Anchor::Right | Anchor::Left);
        layer.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);
        layer.set_exclusive_zone(height);
        layer.set_size(0, height as u32);

        surface.commit();

        (surface, layer)
        //self.base_surface = Some(surface);

        //self.layer_surface = Some(layer);
    }

    fn get_keymap_as_file(&mut self) -> (File, u32) {
        let keymap = self
            .xkb_state
            .get_keymap()
            .get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1);
        let keymap = CString::new(keymap).expect("Keymap should not contain interior nul bytes");
        let keymap = keymap.as_bytes_with_nul();
        let dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let mut file = tempfile::tempfile_in(dir).expect("File could not be created!");
        file.write_all(keymap).unwrap();
        file.flush().unwrap();
        (file, keymap.len() as u32)
    }

    fn init_virtual_keyboard(&mut self, qh: &QueueHandle<Self>) {
        let virtual_keyboard_manager = self.virtual_keyboard_manager.as_ref().unwrap();
        let seat = self.wl_seat.as_ref().unwrap();
        let virtual_keyboard = virtual_keyboard_manager.create_virtual_keyboard(seat, qh, ());
        let (file, size) = self.get_keymap_as_file();
        virtual_keyboard.keymap(
            wl_keyboard::KeymapFormat::XkbV1.into(),
            file.as_raw_fd(),
            size,
        );
        self.virtual_keyboard = Some(virtual_keyboard);
    }

    fn key_press(&self, key: u32) {
        let virtual_keyboard = self.virtual_keyboard.as_ref().unwrap();
        //virtual_keyboard.modifiers(1, 0, 0, 0);
        virtual_keyboard.key(100, key, KeyState::Pressed.into());
    }

    #[must_use]
    fn key_release(&mut self, key: u32) -> bool {
        let virtual_keyboard = self.virtual_keyboard.as_ref().unwrap();
        virtual_keyboard.key(100, key, KeyState::Released.into());
        let mod_pre = self.keymode;
        let keymod: KeyModifierType = key.into();
        self.keymode ^= keymod;
        if self.keymode == mod_pre {
            false
        } else {
            virtual_keyboard.modifiers(self.keymode.bits(), 0, 0, 0);
            true
        }
    }

    // TODO: move it
    fn update_map(&mut self, qh: &QueueHandle<Self>) {
        let key_type = self.keymode;
        let (width, height) = self.keyboard_ui.as_ref().unwrap().get_size();
        let buffer = self.set_buffer(qh, key_type, (width, height));
        let keyboard_ui = self.keyboard_ui.as_mut().unwrap();
        keyboard_ui.update_map(buffer, qh);
    }

    fn init_draw(&mut self, key_type: KeyModifierType, pangoui: &pangoui::PangoUi, tmp: &File) {
        let mut buf = std::io::BufWriter::new(tmp);

        for index in pangoui.ui(key_type).pixels() {
            buf.write_all(&index.0).unwrap();
        }
        buf.flush().unwrap();
    }
    fn draw(&mut self, key_type: KeyModifierType, tmp: &File) {
        let mut buf = std::io::BufWriter::new(tmp);

        for index in self.keyboard_ui.as_ref().unwrap().ui(key_type).pixels() {
            buf.write_all(&index.0).unwrap();
        }
        buf.flush().unwrap();
    }

    fn get_key_point(&self) -> Option<u32> {
        self.keyboard_ui.as_ref().unwrap().get_key_point()
    }

    fn get_key_touch(&self) -> Option<u32> {
        self.keyboard_ui.as_ref().unwrap().get_key_touch()
    }
}
