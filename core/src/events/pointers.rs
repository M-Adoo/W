use super::CommonEvent;
use crate::{
  impl_all_event, impl_common_event_deref, impl_compose_child_for_listener, impl_listener,
  impl_multi_event_listener, prelude::*,
};
use rxrust::prelude::*;
use std::{
  convert::Infallible,
  time::{Duration, Instant},
};
mod from_mouse;
const MULTI_TAP_DURATION: Duration = Duration::from_millis(250);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerId(usize);

/// The pointer is a hardware-agnostic device that can target a specific set of
/// screen coordinates. Having a single event model for pointers can simplify
/// creating Web sites and applications and provide a good user experience
/// regardless of the user's hardware. However, for scenarios when
/// device-specific handling is desired, pointer events defines a pointerType
/// property to inspect the device type which produced the event.
/// Reference: <https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events#term_pointer_event>
#[derive(Debug)]
pub struct PointerEvent {
  /// A unique identifier for the pointer causing the event.
  pub id: PointerId,
  /// The width (magnitude on the X axis), in pixels, of the contact geometry of
  /// the pointer.
  pub width: f32,
  /// the height (magnitude on the Y axis), in pixels, of the contact geometry
  /// of the pointer.
  pub height: f32,
  /// the normalized pressure of the pointer input in the range of 0 to 1, where
  /// 0 and 1 represent the minimum and maximum pressure the hardware is capable
  /// of detecting, respectively. tangentialPressure
  /// The normalized tangential pressure of the pointer input (also known as
  /// barrel pressure or cylinder stress) in the range -1 to 1, where 0 is the
  /// neutral position of the control.
  pub pressure: f32,
  /// The plane angle (in degrees, in the range of -90 to 90) between the Y–Z
  /// plane and the plane containing both the pointer (e.g. pen stylus) axis and
  /// the Y axis.
  pub tilt_x: f32,
  /// The plane angle (in degrees, in the range of -90 to 90) between the X–Z
  /// plane and the plane containing both the pointer (e.g. pen stylus) axis and
  /// the X axis.
  pub tilt_y: f32,
  /// The clockwise rotation of the pointer (e.g. pen stylus) around its major
  /// axis in degrees, with a value in the range 0 to 359.
  pub twist: f32,
  ///  Indicates the device type that caused the event (mouse, pen, touch, etc.)
  pub point_type: PointerType,
  /// Indicates if the pointer represents the primary pointer of this pointer
  /// type.
  pub is_primary: bool,

  pub common: CommonEvent,
}

bitflags! {
  #[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
  pub struct MouseButtons: u8 {
    /// Primary button (usually the left button)
    const PRIMARY = 0b0000_0001;
    /// Secondary button (usually the right button)
    const SECONDARY = 0b0000_0010;
    /// Auxiliary button (usually the mouse wheel button or middle button)
    const AUXILIARY = 0b0000_0100;
    /// 4th button (typically the "Browser Back" button)
    const FOURTH = 0b0000_1000;
    /// 5th button (typically the "Browser Forward" button)
    const FIFTH = 0b0001_0000;
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerType {
  /// The event was generated by a mouse device.
  Mouse,
  /// The event was generated by a pen or stylus device.
  Pen,
  /// The event was generated by a touch, such as a finger.
  Touch,
}

impl_multi_event_listener!(
  "The listener use to fire and listen pointer events.",
  Pointer,
  "",
  PointerDown,
  "",
  PointerDownCapture,
  "",
  PointerUp,
  "",
  PointerUpCapture,
  "",
  PointerMove,
  "",
  PointerMoveCapture,
  "",
  PointerCancel,
  "",
  PointerEnter,
  "",
  PointerLeave,
  "",
  Tap,
  "",
  TapCapture
);

impl_common_event_deref!(PointerEvent);

pub type PointerSubject = MutRefItemSubject<'static, AllPointer, Infallible>;

impl_compose_child_for_listener!(PointerListener);

fn x_times_tap_map_filter(
  x: usize,
  dur: Duration,
  capture: bool,
) -> impl FnMut(&mut AllPointer) -> Option<&mut PointerEvent> {
  assert!(x > 0);
  struct TapInfo {
    pointer_id: PointerId,
    stamps: Vec<Instant>,
  }

  let mut type_info: Option<TapInfo> = None;
  move |e: &mut AllPointer| {
    let e = match e {
      AllPointer::Tap(e) if !capture => e,
      AllPointer::TapCapture(e) if capture => e,
      _ => return None,
    };
    let now = Instant::now();
    match &mut type_info {
      Some(info) if info.pointer_id == e.id => {
        if info.stamps.len() + 1 == x {
          if now.duration_since(info.stamps[0]) <= dur {
            // emit x-tap event and reset the tap info
            type_info = None;
            Some(e)
          } else {
            // remove the expired tap
            info.stamps.remove(0);
            info.stamps.push(now);
            None
          }
        } else {
          info.stamps.push(now);
          None
        }
      }
      _ => {
        type_info = Some(TapInfo { pointer_id: e.id, stamps: vec![now] });
        None
      }
    }
  }
}

impl PointerListener {
  pub fn on_double_tap(self, handler: impl FnMut(&mut PointerEvent) + 'static) -> Self {
    self.on_x_times_tap((2, handler))
  }

  pub fn on_double_tap_capture(self, handler: impl FnMut(&mut PointerEvent) + 'static) -> Self {
    self.on_x_times_tap_capture((2, handler))
  }

  pub fn on_triple_tap(self, handler: impl FnMut(&mut PointerEvent) + 'static) -> Self {
    self.on_x_times_tap((3, handler))
  }

  pub fn on_triple_tap_capture(self, handler: impl FnMut(&mut PointerEvent) + 'static) -> Self {
    self.on_x_times_tap_capture((3, handler))
  }

  pub fn on_x_times_tap(
    self,
    (times, handler): (usize, impl FnMut(&mut PointerEvent) + 'static),
  ) -> Self {
    self.on_x_times_tap_impl(times, MULTI_TAP_DURATION, false, handler)
  }

  pub fn on_x_times_tap_capture(
    self,
    (times, handler): (usize, impl FnMut(&mut PointerEvent) + 'static),
  ) -> Self {
    self.on_x_times_tap_impl(times, MULTI_TAP_DURATION, true, handler)
  }

  fn on_x_times_tap_impl(
    mut self,
    times: usize,
    dur: Duration,
    capture: bool,
    handler: impl FnMut(&mut PointerEvent) + 'static,
  ) -> Self {
    self
      .subject()
      .filter_map(x_times_tap_map_filter(times, dur, capture))
      .subscribe(handler);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    reset_test_env,
    test_helper::{MockBox, MockMulti, TestWindow},
  };
  use std::{cell::RefCell, rc::Rc};
  use winit::{
    dpi::LogicalPosition,
    event::{DeviceId, ElementState, MouseButton, WindowEvent},
  };

  fn tap_on(wnd: &Window, x: f32, y: f32) {
    let device_id = unsafe { DeviceId::dummy() };
    let logical = LogicalPosition::new(x, y);
    #[allow(deprecated)]
    wnd.processes_native_event(WindowEvent::CursorMoved {
      device_id,
      position: logical.to_physical(1.),
      modifiers: ModifiersState::default(),
    });
    #[allow(deprecated)]
    wnd.processes_native_event(WindowEvent::MouseInput {
      device_id,
      state: ElementState::Pressed,
      button: MouseButton::Left,
      modifiers: ModifiersState::default(),
    });
    #[allow(deprecated)]
    wnd.processes_native_event(WindowEvent::MouseInput {
      device_id,
      state: ElementState::Released,
      button: MouseButton::Left,
      modifiers: ModifiersState::default(),
    });
  }

  #[test]
  fn tap_focus() {
    reset_test_env!();

    let tap_cnt = Rc::new(RefCell::new(0));
    let is_focused = Rc::new(RefCell::new(false));

    let tap_cnt1 = tap_cnt.clone();
    let tap_cnt2 = tap_cnt.clone();
    let is_focused2 = is_focused.clone();
    let w = fn_widget! {
      let mut host = @MockMulti {};
      watch!($host.has_focus())
        .subscribe(move |v| *is_focused2.borrow_mut() = v);

      @$host {
        @MockBox {
          size: Size::new(50., 50.,),
          on_tap: move |_| *tap_cnt1.borrow_mut() += 1,
        }
        @MockBox {
          size: Size::new(50., 50.,),
          on_tap: move |_| *tap_cnt2.borrow_mut() += 1,
          on_key_down: move |_| println!("dummy code"),
        }
      }
    };
    let mut wnd = TestWindow::new_with_size(w, Size::new(100., 100.));
    wnd.draw_frame();

    tap_on(&wnd, 25., 25.);
    wnd.draw_frame();
    assert_eq!(*tap_cnt.borrow(), 1);
    assert!(!*is_focused.borrow());

    tap_on(&wnd, 75., 25.);
    wnd.draw_frame();
    assert_eq!(*tap_cnt.borrow(), 2);
    assert!(*is_focused.borrow());
  }
}
