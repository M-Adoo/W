use crate::prelude::*;
use std::{cell::Cell, rc::Rc};
use winit::window::CursorIcon;

/// `Cursor` is an attribute to assign an `cursor` to a widget.
#[derive(Debug)]
pub struct Cursor(Rc<Cell<CursorIcon>>);

pub fn cursor_attach<W>(icon: CursorIcon, w: W) -> W::Target
where
  W: AttachAttr,
{
  let mut default = false;
  let w = w.inspect_or_else(
    || {
      default = true;
      Cursor::new(icon)
    },
    |attr| attr.set_icon(icon),
  );
  if !default {
    return w;
  }

  w.on_pointer_move(move |e| {
    let mut ctx = e.context();
    if e.point_type == PointerType::Mouse
      && e.buttons == MouseButtons::empty()
      && ctx.updated_cursor().is_none()
    {
      if let Some(icon) = ctx.find_attr::<Cursor>().map(|c| c.0.get()) {
        ctx.set_cursor(icon);
      }
    }
  })
}

impl Cursor {
  pub fn new(icon: CursorIcon) -> Self { Cursor(Rc::new(Cell::new(icon))) }

  #[inline]
  pub fn icon(&self) -> CursorIcon { self.0.get() }

  #[inline]
  pub fn set_icon(&self, icon: CursorIcon) { self.0.set(icon) }
}

impl Default for Cursor {
  #[inline]
  fn default() -> Self { Self::new(CursorIcon::Default) }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    prelude::window::Window,
    widget::window::{MockRawWindow, RawWindow},
  };
  use winit::event::{DeviceId, WindowEvent};

  fn submit_cursor(wnd: &mut Window) -> CursorIcon {
    let ptr = (&mut **wnd.raw_window.borrow_mut()) as *mut dyn RawWindow;
    #[allow(clippy::cast_ptr_alignment)]
    let mock_window = unsafe { &mut *(ptr as *mut MockRawWindow) };
    let cursor = mock_window.cursor.unwrap();
    mock_window.set_cursor();
    cursor
  }

  #[test]
  fn tree_down_up() {
    let row_tree = declare! {
      SizedBox{
        size: Size::new(f32::INFINITY, f32::INFINITY),
        cursor: CursorIcon::AllScroll,
        Row{
          cross_align: CrossAxisAlign::Start,
          main_align: MainAxisAlign::Start,
          ..<_>::default(),
          SizedBox {
            size: Size::new(200., 200.),
            cursor: CursorIcon::Hand,
            Row {
              cross_align: CrossAxisAlign::Start,
              main_align: MainAxisAlign::Start,
              ..<_>::default(),
              SizedBox {
                size:  Size::new(100., 100.),
                cursor: CursorIcon::Help,
              }
            }
          }
        }
      }
    };
    let mut wnd = Window::without_render(row_tree, Size::new(400., 400.));

    wnd.render_ready();

    let device_id = unsafe { DeviceId::dummy() };
    wnd.dispatcher.dispatch(WindowEvent::CursorMoved {
      device_id,
      position: (1, 1).into(),
      modifiers: ModifiersState::default(),
    });
    assert_eq!(submit_cursor(&mut wnd), CursorIcon::Help);

    let device_id = unsafe { DeviceId::dummy() };
    wnd.dispatcher.dispatch(WindowEvent::CursorMoved {
      device_id,
      position: (101, 1).into(),
      modifiers: ModifiersState::default(),
    });
    assert_eq!(submit_cursor(&mut wnd), CursorIcon::Hand);

    let device_id = unsafe { DeviceId::dummy() };
    wnd.dispatcher.dispatch(WindowEvent::CursorMoved {
      device_id,
      position: (201, 1).into(),
      modifiers: ModifiersState::default(),
    });
    assert_eq!(submit_cursor(&mut wnd), CursorIcon::AllScroll);

    let device_id = unsafe { DeviceId::dummy() };
    wnd.dispatcher.dispatch(WindowEvent::CursorMoved {
      device_id,
      position: (101, 1).into(),
      modifiers: ModifiersState::default(),
    });
    assert_eq!(submit_cursor(&mut wnd), CursorIcon::Hand);

    let device_id = unsafe { DeviceId::dummy() };
    wnd.dispatcher.dispatch(WindowEvent::CursorMoved {
      device_id,
      position: (1, 1).into(),
      modifiers: ModifiersState::default(),
    });
    assert_eq!(submit_cursor(&mut wnd), CursorIcon::Help);
  }
}
