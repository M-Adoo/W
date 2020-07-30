use super::{add_listener, dispatch_event, EventCommon};
use crate::{prelude::*, widget::inherit_widget};
mod from_mouse;
#[derive(Debug, Clone)]
pub struct PointerId(usize);

/// The pointer is a hardware-agnostic device that can target a specific set of
/// screen coordinates. Having a single event model for pointers can simplify
/// creating Web sites and applications and provide a good user experience
/// regardless of the user's hardware. However, for scenarios when
/// device-specific handling is desired, pointer events defines a pointerType
/// property to inspect the device type which produced the event.
/// Reference: https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events#term_pointer_event
#[derive(Debug, Clone)]
pub struct PointerEvent {
  /// The X, Y coordinate of the pointer in current target widget.
  pub position: Point,
  // The X, Y coordinate of the mouse pointer in global (window) coordinates.
  pub global_pos: Point,
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
  /// The buttons being depressed (if any) when the mouse event was fired.
  pub buttons: MouseButtons,
  common: EventCommon,
}

bitflags! {
  #[derive(Default)]
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

impl std::convert::AsRef<EventCommon> for PointerEvent {
  #[inline]
  fn as_ref(&self) -> &EventCommon { &self.common }
}

impl std::convert::AsMut<EventCommon> for PointerEvent {
  #[inline]
  fn as_mut(&mut self) -> &mut EventCommon { &mut self.common }
}

impl PointerEvent {
  /// The button number that was pressed (if applicable) when the mouse event
  /// was fired.
  pub fn button_num(&self) -> u32 { self.buttons.bits().count_ones() }
}

/// A widget that calls callbacks in response to common pointer events.
pub struct PointerListener {
  widget: BoxWidget,
  on_pointer_down: Option<Box<dyn FnMut(&PointerEvent)>>,
  on_pointer_move: Option<Box<dyn FnMut(&PointerEvent)>>,
  on_pointer_up: Option<Box<dyn FnMut(&PointerEvent)>>,
  on_pointer_cancel: Option<Box<dyn FnMut(&PointerEvent)>>,
  on_pointer_enter: Option<Box<dyn FnMut(&PointerEvent)>>,
  on_pointer_leave: Option<Box<dyn FnMut(&PointerEvent)>>,
}

/// Enter/Leave do not bubble.
pub enum PointerEventType {
  Down,
  Move,
  Up,
  Cancel,
  Enter,
  Leave,
  /* onpointerover:
   * onpointerout:
   * gotpointercapture:
   * lostpointercapture: */
}

impl PointerListener {
  pub fn listen_on<H: FnMut(&PointerEvent) + 'static>(
    base: BoxWidget,
    event_type: PointerEventType,
    handler: H,
  ) -> BoxWidget {
    let mut pointer = inherit(
      base,
      |base| Self {
        widget: base,
        on_pointer_down: None,
        on_pointer_move: None,
        on_pointer_up: None,
        on_pointer_cancel: None,
        on_pointer_enter: None,
        on_pointer_leave: None,
      },
      |_| {},
    );
    Widget::dynamic_cast_mut::<Self>(&mut pointer)
      .unwrap()
      .add_listener(event_type, handler);
    pointer
  }

  fn add_listener<F: FnMut(&PointerEvent) + 'static>(
    &mut self,
    event_type: PointerEventType,
    handler: F,
  ) {
    let holder = self.pointer_handler(event_type);
    add_listener(holder, handler);
  }

  pub(crate) fn dispatch(&mut self, event_type: PointerEventType, event: &PointerEvent) {
    let handler = self.pointer_handler(event_type);
    dispatch_event(handler, event)
  }

  fn pointer_handler(
    &mut self,
    event_type: PointerEventType,
  ) -> &mut Option<Box<dyn FnMut(&PointerEvent)>> {
    match event_type {
      PointerEventType::Down => &mut self.on_pointer_down,
      PointerEventType::Move => &mut self.on_pointer_move,
      PointerEventType::Up => &mut self.on_pointer_up,
      PointerEventType::Cancel => &mut self.on_pointer_cancel,
      PointerEventType::Enter => &mut self.on_pointer_enter,
      PointerEventType::Leave => &mut self.on_pointer_leave,
    }
  }
}

impl std::fmt::Debug for PointerListener {
  fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // Todo: we should remove the `Debug` bound for widget.
    Ok(())
  }
}

inherit_widget!(PointerListener, widget);
