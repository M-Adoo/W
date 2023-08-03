use indextree::{Arena, Node, NodeId};
use rxrust::prelude::*;

use super::WidgetTree;
use crate::{
  builtin_widgets::Void,
  context::{PaintingCtx, WidgetCtx},
  state::{ModifyScope, StateChangeNotifier},
  widget::{QueryOrder, Render},
  window::DelayEvent,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Hash)]

pub struct WidgetId(pub(crate) NodeId);

pub(crate) type TreeArena = Arena<RenderNode>;

bitflags! {
  #[derive(Default, PartialEq, Eq, Clone, Copy, Hash, Debug)]
  pub(crate) struct  RenderNodeFlag: u8 {
    const NONE = 0;
    const DROPPED = 0b0001;
  }
}
pub(crate) struct RenderNode {
  flags: RenderNodeFlag,
  data: Box<dyn Render>,
}

impl WidgetId {
  /// Returns a reference to the node data.
  pub(crate) fn get(self, tree: &TreeArena) -> Option<&dyn Render> {
    self.get_node(tree).map(|n| n.data.as_ref())
  }

  /// Returns a mutable reference to the node data.
  pub(crate) fn get_mut(self, tree: &mut TreeArena) -> Option<&mut Box<dyn Render>> {
    self.get_node_mut(tree).map(|n| &mut n.data)
  }

  pub(crate) fn get_node(self, tree: &TreeArena) -> Option<&RenderNode> {
    tree.get(self.0).map(|n| n.get())
  }

  pub(crate) fn get_node_mut(self, tree: &mut TreeArena) -> Option<&mut RenderNode> {
    tree.get_mut(self.0).map(|n| n.get_mut())
  }

  /// Mark the widget dropped but not release the node, caller has the
  /// responsibility to release it.
  pub(crate) fn mark_drop(self, tree: &mut TreeArena) {
    if let Some(node) = self.get_node_mut(tree) {
      node.flags.insert(RenderNodeFlag::DROPPED);
    }
  }

  /// detect if the widget of this id point to is dropped.
  pub(crate) fn is_dropped(self, tree: &TreeArena) -> bool {
    self.0.is_removed(tree)
      || self
        .get_node(tree)
        .map_or(true, |n| n.flags.contains(RenderNodeFlag::DROPPED))
  }

  #[allow(clippy::needless_collect)]
  pub(crate) fn lowest_common_ancestor(
    self,
    other: WidgetId,
    tree: &TreeArena,
  ) -> Option<WidgetId> {
    self.common_ancestors(other, tree).last()
  }

  #[allow(clippy::needless_collect)]
  // return ancestors from root to lowest common ancestor
  pub(crate) fn common_ancestors(
    self,
    other: WidgetId,
    tree: &TreeArena,
  ) -> impl Iterator<Item = WidgetId> + '_ {
    let mut p0 = vec![];
    let mut p1 = vec![];
    if !self.is_dropped(tree) && !other.is_dropped(tree) {
      p0 = other.ancestors(tree).collect::<Vec<_>>();
      p1 = self.ancestors(tree).collect::<Vec<_>>();
    }

    p0.into_iter()
      .rev()
      .zip(p1.into_iter().rev())
      .take_while(|(a, b)| a == b)
      .map(|(a, _)| a)
  }

  pub(crate) fn parent(self, tree: &TreeArena) -> Option<WidgetId> {
    self.node_feature(tree, |node| node.parent())
  }

  pub(crate) fn first_child(self, tree: &TreeArena) -> Option<WidgetId> {
    self.node_feature(tree, |node| node.first_child())
  }

  pub(crate) fn last_child(self, tree: &TreeArena) -> Option<WidgetId> {
    self.node_feature(tree, |node| node.last_child())
  }

  pub(crate) fn next_sibling(self, tree: &TreeArena) -> Option<WidgetId> {
    self.node_feature(tree, |node| node.next_sibling())
  }
  pub(crate) fn prev_sibling(self, tree: &TreeArena) -> Option<WidgetId> {
    self.node_feature(tree, Node::previous_sibling)
  }

  pub(crate) fn previous_sibling(self, tree: &TreeArena) -> Option<WidgetId> {
    self.node_feature(tree, |node| node.previous_sibling())
  }

  pub(crate) fn ancestors(self, tree: &TreeArena) -> impl Iterator<Item = WidgetId> + '_ {
    self.0.ancestors(tree).map(WidgetId)
  }

  #[inline]
  pub(crate) fn children(self, arena: &TreeArena) -> impl Iterator<Item = WidgetId> + '_ {
    self.0.children(arena).map(WidgetId)
  }

  pub(crate) fn descendants(self, tree: &TreeArena) -> impl Iterator<Item = WidgetId> + '_ {
    self.0.descendants(tree).map(WidgetId)
  }

  pub(crate) fn on_mounted_subtree(self, tree: &WidgetTree) {
    self
      .descendants(&tree.arena)
      .for_each(|w| w.on_mounted(tree));
  }

  pub(crate) fn on_mounted(self, tree: &WidgetTree) {
    self.assert_get(&tree.arena).query_all_type(
      |notifier: &StateChangeNotifier| {
        let state_changed = tree.dirty_set.clone();
        notifier
          .raw_modifies()
          .filter(|b| b.contains(ModifyScope::FRAMEWORK))
          .subscribe(move |_| {
            state_changed.borrow_mut().insert(self);
          });
        true
      },
      QueryOrder::OutsideFirst,
    );

    tree.window().add_delay_event(DelayEvent::Mounted(self));
  }

  pub(crate) fn insert_after(self, next: WidgetId, tree: &mut TreeArena) {
    self.0.insert_after(next.0, tree);
  }

  pub(crate) fn insert_before(self, before: WidgetId, tree: &mut TreeArena) {
    self.0.insert_before(before.0, tree);
  }

  pub(crate) fn append(self, child: WidgetId, tree: &mut TreeArena) {
    self.0.append(child.0, tree);
  }

  /// Return the single child of `widget`, panic if have more than once child.
  pub(crate) fn single_child(&self, tree: &TreeArena) -> Option<WidgetId> {
    assert_eq!(
      self.first_child(tree),
      self.last_child(tree),
      "Have more than one child."
    );
    self.first_child(tree)
  }

  fn node_feature(
    self,
    tree: &TreeArena,
    method: impl FnOnce(&Node<RenderNode>) -> Option<NodeId>,
  ) -> Option<WidgetId> {
    tree.get(self.0).and_then(method).map(WidgetId)
  }

  pub(crate) fn assert_get(self, tree: &TreeArena) -> &dyn Render {
    self.get(tree).expect("Widget not exists in the `tree`")
  }

  pub(crate) fn assert_get_mut(self, tree: &mut TreeArena) -> &mut Box<dyn Render> {
    self.get_mut(tree).expect("Widget not exists in the `tree`")
  }

  pub(crate) fn paint_subtree(self, ctx: &mut PaintingCtx) {
    let mut w = Some(self);
    while let Some(id) = w {
      ctx.id = id;
      ctx.painter.save();
      let wnd = ctx.window();
      let arena = &wnd.widget_tree.borrow().arena;

      let mut need_paint = false;
      if ctx.painter.alpha() != 0. {
        if let Some(layout_box) = ctx.box_rect() {
          let render = id.assert_get(arena);
          ctx
            .painter
            .translate(layout_box.min_x(), layout_box.min_y());
          render.paint(ctx);
          need_paint = true;
        }
      }

      w = id.first_child(arena).filter(|_| need_paint).or_else(|| {
        let mut node = w;
        while let Some(p) = node {
          // self node sub-tree paint finished, goto sibling
          ctx.painter.restore();
          node = match p == self {
            true => None,
            false => p.next_sibling(arena),
          };
          if node.is_some() {
            break;
          } else {
            // if there is no more sibling, back to parent to find sibling.
            node = p.parent(arena);
          }
        }
        node
      });
    }
  }
}

pub(crate) unsafe fn split_arena(tree: &mut TreeArena) -> (&mut TreeArena, &mut TreeArena) {
  let ptr = tree as *mut TreeArena;
  (&mut *ptr, &mut *ptr)
}

pub(crate) fn new_node(arena: &mut TreeArena, node: Box<dyn Render>) -> WidgetId {
  WidgetId(arena.new_node(RenderNode {
    flags: RenderNodeFlag::NONE,
    data: node,
  }))
}

pub(crate) fn empty_node(arena: &mut TreeArena) -> WidgetId { new_node(arena, Box::new(Void)) }

impl std::ops::Deref for RenderNode {
  type Target = dyn Render;
  fn deref(&self) -> &Self::Target { self.data.as_ref() }
}

impl std::ops::DerefMut for RenderNode {
  fn deref_mut(&mut self) -> &mut Self::Target { self.data.as_mut() }
}
