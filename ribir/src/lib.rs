pub use ribir_core as core;
#[cfg(feature = "widgets")]
pub use ribir_widgets as widgets;
pub mod app;
pub mod timer;
mod winit_shell_wnd;

#[cfg(feature = "material")]
pub use ribir_material as material;

pub mod prelude {
  #[cfg(feature = "material")]
  pub use super::material;
  #[cfg(feature = "widgets")]
  pub use super::widgets::prelude::*;
  pub use crate::app::*;
  pub use ribir_core::prelude::*;
}
