use crate::{impl_query_self_only, prelude::*};
use ribir_geom::{Rect, Size};

impl Render for ShareResource<PixelImage> {
  fn perform_layout(&self, _: BoxClamp, _: &mut LayoutCtx) -> Size {
    Size::new(self.width() as f32, self.height() as f32)
  }

  fn paint(&self, ctx: &mut PaintingCtx) {
    let size = ctx.box_size().unwrap();
    let box_rect = Rect::from_size(size);
    let img_rect = Rect::from_size(Size::new(self.width() as f32, self.height() as f32));
    let painter = ctx.painter();
    if let Some(rc) = img_rect.intersection(&box_rect) {
      painter.draw_img(self.clone(), &rc, &Some(rc));
    }
  }
}

impl Query for ShareResource<PixelImage> {
  impl_query_self_only!();
}
