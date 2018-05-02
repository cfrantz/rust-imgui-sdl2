extern crate sdl2;
extern crate imgui;

use sdl2::sys as sdl2_sys;
use imgui::sys as imgui_sys;

use sdl2::video::Window;
use sdl2::EventPump;
// use sdl2::mouse::{Cursor,SystemCursor};
use imgui::ImGui;
use std::time::Instant;
use std::os::raw::{c_char, c_void};

use sdl2::event::Event;

pub struct ImguiSdl2<'a> {
  window: &'a Window,
  last_frame: Instant,
  mouse_press: [bool; 5],
}

impl<'a> ImguiSdl2<'a> {
  pub fn new(
    window: &'a Window,
    imgui: &mut ImGui,
  ) -> Self {


    // TODO: upstream to imgui-rs
    {
      let io = unsafe { &mut *imgui_sys::igGetIO() };

      io.get_clipboard_text_fn = Some(get_clipboard_text);
      io.set_clipboard_text_fn = Some(set_clipboard_text);
      io.clipboard_user_data = std::ptr::null_mut();
    }

    {
      use sdl2::keyboard::Scancode;
      use imgui::ImGuiKey;

      imgui.set_imgui_key(ImGuiKey::Tab, Scancode::Tab as u8);
      imgui.set_imgui_key(ImGuiKey::LeftArrow, Scancode::Left as u8);
      imgui.set_imgui_key(ImGuiKey::RightArrow, Scancode::Right as u8);
      imgui.set_imgui_key(ImGuiKey::UpArrow, Scancode::Up as u8);
      imgui.set_imgui_key(ImGuiKey::DownArrow, Scancode::Down as u8);
      imgui.set_imgui_key(ImGuiKey::PageUp, Scancode::PageUp as u8);
      imgui.set_imgui_key(ImGuiKey::PageDown, Scancode::PageDown as u8);
      imgui.set_imgui_key(ImGuiKey::Home, Scancode::Home as u8);
      imgui.set_imgui_key(ImGuiKey::End, Scancode::End as u8);
      imgui.set_imgui_key(ImGuiKey::Delete, Scancode::Delete as u8);
      imgui.set_imgui_key(ImGuiKey::Backspace, Scancode::Backspace as u8);
      imgui.set_imgui_key(ImGuiKey::Enter, Scancode::Return as u8);
      imgui.set_imgui_key(ImGuiKey::Escape, Scancode::Escape as u8);
      imgui.set_imgui_key(ImGuiKey::A, Scancode::A as u8);
      imgui.set_imgui_key(ImGuiKey::C, Scancode::C as u8);
      imgui.set_imgui_key(ImGuiKey::V, Scancode::V as u8);
      imgui.set_imgui_key(ImGuiKey::X, Scancode::X as u8);
      imgui.set_imgui_key(ImGuiKey::Y, Scancode::Y as u8);
      imgui.set_imgui_key(ImGuiKey::Z, Scancode::Z as u8);
    }

    Self {
      window,
      last_frame: Instant::now(),
      mouse_press: [false; 5],
    }
  }

  pub fn handle_event(&mut self, imgui: &mut ImGui, event: &Event) {
    use sdl2::mouse::MouseButton;
    use sdl2::keyboard;

    fn set_mod(imgui: &mut ImGui, keymod: keyboard::Mod) {
      let ctrl = keymod.intersects(keyboard::RCTRLMOD | keyboard::LCTRLMOD);
      let alt = keymod.intersects(keyboard::RALTMOD | keyboard::LALTMOD);
      let shift = keymod.intersects(keyboard::RSHIFTMOD | keyboard::LSHIFTMOD);
      let super_ = keymod.intersects(keyboard::RGUIMOD | keyboard::LGUIMOD);

      imgui.set_key_ctrl(ctrl);
      imgui.set_key_alt(alt);
      imgui.set_key_shift(shift);
      imgui.set_key_super(super_);
    }

    match event {
      &Event::MouseWheel{y, ..} => {
        imgui.set_mouse_wheel(y as f32);
      },
      &Event::MouseButtonDown{mouse_btn, ..} => {
        if mouse_btn != MouseButton::Unknown {
          let index = match mouse_btn {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::X1 => 3,
            MouseButton::X2 => 4,
            MouseButton::Unknown => unreachable!(),
          };
          self.mouse_press[index] = true;
        }
      },
      &Event::TextInput{ref text, .. } => {
        for chr in text.chars() {
          imgui.add_input_character(chr);
        }
      },
      &Event::KeyDown{scancode, keymod, .. } => {
        set_mod(imgui, keymod);
        if let Some(scancode) = scancode {
          imgui.set_key(scancode as u8, true);
        }
      },
      &Event::KeyUp{scancode, keymod, .. } => {
        set_mod(imgui, keymod);
        if let Some(scancode) = scancode {
          imgui.set_key(scancode as u8, false);
        }
      },
      _ => {},
    }
  }

  pub fn frame<'ui>(
    &mut self,
    imgui: &'ui mut ImGui,
    event_pump: &EventPump,
  ) -> imgui::Ui<'ui> {
    // let mouse_util = self.window.subsystem().sdl().mouse();

    // Merging the mousedown events we received into the current state prevents us from missing
    // clicks that happen faster than a frame
    let mouse_state = event_pump.mouse_state();
    let mouse_down = [
      self.mouse_press[0] || mouse_state.left(),
      self.mouse_press[1] || mouse_state.right(),
      self.mouse_press[2] || mouse_state.middle(),
      self.mouse_press[3] || mouse_state.x1(),
      self.mouse_press[4] || mouse_state.x2(),
    ];
    imgui.set_mouse_down(&mouse_down);
    self.mouse_press = [false; 5];

    // TODO: SDL2 0.31
    // let any_mouse_down = mouse_down.iter().any(|b| b);
    // mouse_util.capture(any_mouse_down);


    imgui.set_mouse_pos(mouse_state.x() as f32, mouse_state.y() as f32);


    // TODO: imgui 0.19
    // let mouse_cursor = imgui.mouse_cursor();
    // if imgui.mouse_draw_cursor() || mouse_cursor == ImGuiMouseCursor::None {
    //   mouse_util.show_cursor(false);
    // } else {
    //   mouse_util.show_cursor(true);

    //   let sdl_cursor = match mouse_cursor {
    //     ImGuiMouseCursor::None => unreachable!("mouse_cursor was None!"),
    //     ImGuiMouseCursor::Arrow => SystemCursor::Arrow,
    //     ImGuiMouseCursor::TextInput => SystemCursor::IBeam,
    //     ImGuiMouseCursor::Move => SystemCursor::SizeAll,
    //     ImGuiMouseCursor::ResizeNS => SystemCursor::SizeNS,
    //     ImGuiMouseCursor::ResizeEW => SystemCursor::SizeWE,
    //     ImGuiMouseCursor::ResizeNESW => SystemCursor::SizeNESW,
    //     ImGuiMouseCursor::ResizeNWSE => SystemCursor::SizeNWSE,
    //   };

    //   Cursor::from_system(sdl_cursor).unwrap().set();
    // }




    let now = Instant::now();
    let delta = now - self.last_frame;
    let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
    self.last_frame = now;

    let window_size = self.window.size();
    let display_size = self.window.drawable_size();

    let ui = imgui.frame(window_size, display_size, delta_s);

    ui
  }
}

pub extern "C" fn get_clipboard_text(_user_data: *mut c_void) -> *const c_char {
  unsafe { sdl2_sys::SDL_GetClipboardText() }
}

pub extern "C" fn set_clipboard_text(_user_data: *mut c_void, text: *const c_char) {
  unsafe { sdl2_sys::SDL_SetClipboardText(text) };
}