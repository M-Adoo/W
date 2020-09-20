#![feature(min_specialization, test, decl_macro, negative_impls, drain_filter)]

#[macro_use]
extern crate bitflags;

mod application;
mod render;
mod util;
pub mod widget;
pub mod prelude {
  pub use crate::application::Application;
  pub use crate::render::*;
  pub use crate::widget;
  pub use crate::widget::{build_ctx::BuildCtx, widget_tree::WidgetId, *};
  pub use canvas::{DevicePoint, DeviceRect, DeviceSize, Point, Rect, Size, Transform, Vector};
}

#[cfg(test)]
pub mod test;
