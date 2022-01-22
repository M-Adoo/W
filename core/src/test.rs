pub mod embed_post;
pub mod key_embed_post;
pub mod recursive_row;

use crate::prelude::*;

// return the flex box rect, and rect of its children.
pub fn widget_and_its_children_box_rect(root: BoxedWidget, window_size: Size) -> (Rect, Vec<Rect>) {
  let mut wnd = window::Window::without_render(root, window_size);
  wnd.render_ready();

  root_and_children_rect(&mut wnd)
}

pub fn root_and_children_rect(wnd: &window::Window) -> (Rect, Vec<Rect>) {
  let ctx = wnd.context();
  let r_tree = &ctx.render_tree;
  let layout = &ctx.layout_store;
  let root = r_tree.root().unwrap();
  let rect = layout.layout_box_rect(root).unwrap();
  let children_box_rect = root
    .children(&*r_tree)
    .map(|rid| layout.layout_box_rect(rid).unwrap())
    .collect();

  (rect, children_box_rect)
}
