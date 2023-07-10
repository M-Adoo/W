use rxrust::ops::filter_map::FilterMapOp;

use super::EventCommon;
use crate::{
  data_widget::compose_child_as_data_widget, impl_compose_child_for_listener, impl_listener,
  impl_listener_and_compose_child, impl_query_self_only, prelude::*,
};
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
#[derive(Debug, Clone)]
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

  pub common: EventCommon,
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

impl std::borrow::Borrow<EventCommon> for PointerEvent {
  #[inline]
  fn borrow(&self) -> &EventCommon { &self.common }
}

impl std::borrow::BorrowMut<EventCommon> for PointerEvent {
  #[inline]
  fn borrow_mut(&mut self) -> &mut EventCommon { &mut self.common }
}

impl std::ops::Deref for PointerEvent {
  type Target = EventCommon;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.common }
}

impl std::ops::DerefMut for PointerEvent {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.common }
}

#[derive(Declare)]
pub struct PointerDownListener {
  #[declare(builtin, convert=custom)]
  on_pointer_down: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct PointerDownCaptureListener {
  #[declare(builtin, convert=custom)]
  on_pointer_down_capture: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct PointerUpListener {
  #[declare(builtin, convert=custom)]
  on_pointer_up: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct PointerUpCaptureListener {
  #[declare(builtin, convert=custom)]
  on_pointer_up_capture: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct PointerMoveListener {
  #[declare(builtin, convert=custom)]
  on_pointer_move: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct PointerMoveCaptureListener {
  #[declare(builtin, convert=custom)]
  on_pointer_move_capture: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct TapListener {
  #[declare(builtin, convert=custom)]
  on_tap: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct TapCaptureListener {
  #[declare(builtin, convert=custom)]
  on_tap_capture: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct PointerCancelListener {
  #[declare(builtin, convert=custom)]
  pub on_pointer_cancel: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct PointerEnterListener {
  #[declare(builtin, convert=custom)]
  on_pointer_enter: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

#[derive(Declare)]
pub struct PointerLeaveListener {
  #[declare(builtin, convert=custom)]
  pub on_pointer_leave: MutRefItemSubject<'static, PointerEvent, Infallible>,
}

impl_listener_and_compose_child!(
  PointerDownListener,
  PointerDownListenerDeclarer,
  on_pointer_down,
  PointerEvent,
  pointer_down_stream
);

impl_listener_and_compose_child!(
  PointerDownCaptureListener,
  PointerDownCaptureListenerDeclarer,
  on_pointer_down_capture,
  PointerEvent,
  pointer_down_capture_stream
);

impl_listener_and_compose_child!(
  PointerUpListener,
  PointerUpListenerDeclarer,
  on_pointer_up,
  PointerEvent,
  pointer_up_stream
);

impl_listener_and_compose_child!(
  PointerUpCaptureListener,
  PointerUpCaptureListenerDeclarer,
  on_pointer_up_capture,
  PointerEvent,
  pointer_up_capture_stream
);

impl_listener_and_compose_child!(
  PointerMoveListener,
  PointerMoveListenerDeclarer,
  on_pointer_move,
  PointerEvent,
  pointer_move_stream
);

impl_listener_and_compose_child!(
  PointerMoveCaptureListener,
  PointerMoveCaptureListenerDeclarer,
  on_pointer_move_capture,
  PointerEvent,
  pointer_move_capture_stream
);

impl_listener_and_compose_child!(
  PointerCancelListener,
  PointerCancelListenerDeclarer,
  on_pointer_cancel,
  PointerEvent,
  pointer_cancel_stream
);

impl_listener_and_compose_child!(
  PointerEnterListener,
  PointerEnterListenerDeclarer,
  on_pointer_enter,
  PointerEvent,
  pointer_enter_stream
);

impl_listener_and_compose_child!(
  PointerLeaveListener,
  PointerLeaveListenerDeclarer,
  on_pointer_leave,
  PointerEvent,
  pointer_leave_stream
);

macro_rules! impl_tap_listener {
  (
    $listener: ident,
    $declarer: ident,
    $x_times_tap: ident,
    $tap: ident,
    $dobule_tap: ident,
    $triple_tap: ident,
    $x_times_tap_stream: ident,
    $tap_stream: ident,
    $double_tap_stream: ident,
    $triple_tap_stream: ident
  ) => {
    impl $declarer {
      pub fn $tap(mut self, handler: impl for<'r> FnMut(&'r mut PointerEvent) + 'static) -> Self {
        self.tap_subject().subscribe(handler);
        self
      }

      pub fn $x_times_tap(
        mut self,
        (times, handler): (usize, impl for<'r> FnMut(&'r mut PointerEvent) + 'static),
      ) -> Self {
        self
          .tap_subject()
          .filter_map(x_times_tap_map_filter(times, MULTI_TAP_DURATION))
          .subscribe(handler);
        self
      }

      pub fn $dobule_tap(
        self,
        handler: impl for<'r> FnMut(&'r mut PointerEvent) + 'static,
      ) -> Self {
        self.$x_times_tap((2, handler))
      }

      pub fn $triple_tap(
        self,
        handler: impl for<'r> FnMut(&'r mut PointerEvent) + 'static,
      ) -> Self {
        self.$x_times_tap((3, handler))
      }

      fn tap_subject(&mut self) -> MutRefItemSubject<'static, PointerEvent, Infallible> {
        self.$tap.get_or_insert_with(Default::default).clone()
      }
    }

    impl Query for $listener {
      impl_query_self_only!();
    }
    impl $listener {
      /// Return an observable stream of this event.
      pub fn $tap_stream(&self) -> MutRefItemSubject<'static, PointerEvent, Infallible> {
        self.$tap.clone()
      }

      /// Return an observable stream of double tap event
      #[inline]
      pub fn $double_tap_stream(
        &self,
      ) -> FilterMapOp<
        MutRefItemSubject<'static, PointerEvent, Infallible>,
        impl FnMut(&mut PointerEvent) -> Option<&mut PointerEvent>,
        &mut PointerEvent,
      > {
        self.$x_times_tap_stream(2, MULTI_TAP_DURATION)
      }

      /// Return an observable stream of tripe tap event
      #[inline]
      pub fn $triple_tap_stream(
        &self,
      ) -> FilterMapOp<
        MutRefItemSubject<'static, PointerEvent, Infallible>,
        impl FnMut(&mut PointerEvent) -> Option<&mut PointerEvent>,
        &mut PointerEvent,
      > {
        self.$x_times_tap_stream(2, MULTI_TAP_DURATION)
      }

      /// Return an observable stream of x-tap event that user tapped 'x' times in
      /// the specify duration `dur`.
      pub fn $x_times_tap_stream(
        &self,
        x: usize,
        dur: Duration,
      ) -> FilterMapOp<
        MutRefItemSubject<'static, PointerEvent, Infallible>,
        impl FnMut(&mut PointerEvent) -> Option<&mut PointerEvent>,
        &mut PointerEvent,
      > {
        self
          .$tap_stream()
          .filter_map(x_times_tap_map_filter(x, dur))
      }
    }
    impl EventListener for $listener {
      type Event = PointerEvent;
      #[inline]
      fn dispatch(&self, event: &mut PointerEvent) { self.$tap.clone().next(event) }
    }

    impl ComposeChild for $listener {
      type Child = Widget;
      #[inline]
      fn compose_child(this: State<Self>, child: Self::Child) -> Widget {
        compose_child_as_data_widget(child, this)
      }
    }
  };
}

fn x_times_tap_map_filter(
  x: usize,
  dur: Duration,
) -> impl FnMut(&mut PointerEvent) -> Option<&mut PointerEvent> {
  assert!(x > 0);
  struct TapInfo {
    pointer_id: PointerId,
    stamps: Vec<Instant>,
  }

  let mut type_info: Option<TapInfo> = None;
  move |e: &mut PointerEvent| {
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

impl_tap_listener!(
  TapListener,
  TapListenerDeclarer,
  on_x_times_tap,
  on_tap,
  on_double_tap,
  on_triple_tap,
  x_times_tap_stream,
  tap_stream,
  dobule_tap_stream,
  triple_tap_stream
);

impl_tap_listener!(
  TapCaptureListener,
  TapCaptureListenerDeclarer,
  on_x_times_tap_capture,
  on_tap_capture,
  on_double_tap_capture,
  on_triple_tap_capture,
  x_times_tap_capture_stream,
  tap_capture_stream,
  double_tap_capture_stream,
  triple_tap_capture_stream
);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_helper::{MockBox, MockMulti, TestWindow};
  use std::{cell::RefCell, rc::Rc};
  use winit::{
    dpi::LogicalPosition,
    event::{DeviceId, ElementState, MouseButton, WindowEvent},
  };

  fn tap_on(wnd: &mut Window, x: f32, y: f32) {
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
    let _guard = unsafe { AppCtx::new_lock_scope() };

    let tap_cnt = Rc::new(RefCell::new(0));
    let is_focused = Rc::new(RefCell::new(false));

    let tap_cnt1 = tap_cnt.clone();
    let tap_cnt2 = tap_cnt.clone();
    let is_focused1 = is_focused.clone();
    let w = widget! {
      MockMulti {
        id: host,
        MockBox {
          size: Size::new(50., 50.,),
          on_tap: move |_| *tap_cnt1.borrow_mut() += 1,
        }
        MockBox {
          size: Size::new(50., 50.,),
          on_tap: move |_| *tap_cnt2.borrow_mut() += 1,
          on_key_down: move |_| println!("dummy code"),
        }
      }
      finally {
        let_watch!(host.has_focus())
          .subscribe(move |v| *is_focused1.borrow_mut() = v);
      }
    };
    let mut wnd = TestWindow::new_with_size(w, Size::new(100., 100.));
    wnd.draw_frame();

    tap_on(&mut wnd, 25., 25.);
    wnd.draw_frame();
    assert_eq!(*tap_cnt.borrow(), 1);
    assert!(!*is_focused.borrow());

    tap_on(&mut wnd, 75., 25.);
    wnd.draw_frame();
    assert_eq!(*tap_cnt.borrow(), 2);
    assert!(*is_focused.borrow());
  }
}
