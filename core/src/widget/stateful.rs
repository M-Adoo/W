use crate::prelude::*;
use rxrust::prelude::*;
use std::{
  cell::{Ref, RefCell, RefMut},
  marker::PhantomData,
  mem::ManuallyDrop,
  pin::Pin,
  ptr::NonNull,
  rc::Rc,
};

/// This widget convert a stateless widget to stateful.
pub type Stateful<W> = WidgetAttr<W, StatefulAttr>;

/// A reference of stateful widget, can use it to directly access and modify
/// stateful widget.
///
/// Remember it assume you changed the widget back of this reference if you
/// mutably borrow this pointer. No matter if you really modify it.
///
/// ## Panics
///
/// `StateRefCell` should not live longer than its widget. Framework guarantee
/// the widgets constructed in the same `build` method  have same lifetime,  and
/// parent live longer than parent. So not pass a `StateRefCell` to its
/// ancestors, that maybe panic.
pub struct StateRefCell<W: Widget> {
  attr: StatefulAttr,
  type_info: PhantomData<*const W>,
}

#[derive(Debug, Clone)]
pub struct StatefulAttr {
  tree: NonNull<widget_tree::WidgetTree>,
  state: InnerState,
  id: WidgetId,
}

#[derive(Debug, Clone)]
struct InnerState {
  change_emitter: Option<LocalSubject<'static, (), ()>>,
  widget_ptr: Rc<RefCell<(*const dyn Widget, *mut dyn Widget)>>,
}

#[derive(Clone)]
pub struct StateChange<T: Clone> {
  before: T,
  after: T,
}

impl<W: Widget> Stateful<W> {
  #[inline]
  pub fn ref_cell(&self) -> StateRefCell<W> { unsafe { self.attr.ref_cell() } }

  #[inline]
  pub fn id(&self) -> WidgetId { self.attr.id }

  /// Event emitted when this widget modified.
  pub fn change_stream(
    &mut self,
  ) -> impl LocalObservable<'static, Item = StateRefCell<W>, Err = ()> {
    let ref_cell = self.ref_cell();
    self
      .attr
      .state
      .change_emitter()
      .map(move |_| ref_cell.clone())
  }

  /// Pick a field change stream from the host widget.
  pub fn pick_state<T: Clone + 'static>(
    &mut self,
    pick: impl Fn(&W) -> T + 'static,
  ) -> impl LocalObservable<'static, Item = StateChange<T>, Err = ()> {
    let v = pick(&*self);
    let init = StateChange {
      before: v.clone(),
      after: v,
    };
    self
      .change_stream()
      .scan_initial(init, move |mut init, value| {
        init.before = init.after;
        init.after = pick(&*value.borrow());
        init
      })
  }
}

impl<W: Widget> StateRefCell<W> {
  #[inline]
  pub fn borrow(&self) -> Ref<W> { unsafe { self.attr.state.borrow() } }

  #[inline]
  pub fn borrow_mut(&mut self) -> StateRefMut<W> {
    StateRefMut {
      attr: self.attr.clone(),
      ref_mut: ManuallyDrop::new(unsafe { self.attr.state.borrow_mut() }),
    }
  }
}

impl<W: Widget> Clone for StateRefCell<W> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      attr: self.attr.clone(),
      type_info: self.type_info,
    }
  }
}

pub struct StateRefMut<'a, W: Widget> {
  ref_mut: ManuallyDrop<RefMut<'a, W>>,
  attr: StatefulAttr,
}

impl<'a, W: Widget> Drop for StateRefMut<'a, W> {
  fn drop(&mut self) {
    let StatefulAttr {
      id, state, tree, ..
    } = &mut self.attr;

    // Safety drop the RefMut first , will never borrow it.
    unsafe { ManuallyDrop::drop(&mut self.ref_mut) };
    if let Some(subject) = &mut state.change_emitter {
      subject.next(());
    }

    unsafe {
      let tree = tree.as_mut();
      id.mark_changed(tree);
    }
  }
}

impl<'a, W: Widget> std::ops::Deref for StateRefMut<'a, W> {
  type Target = RefMut<'a, W>;
  fn deref(&self) -> &Self::Target { &self.ref_mut }
}

impl<'a, W: Widget> std::ops::DerefMut for StateRefMut<'a, W> {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.ref_mut }
}

impl<W: Widget> Stateful<W> {
  pub fn stateful<A: AttributeAttach<HostWidget = W>>(
    widget: A,
    mut tree: Pin<&mut widget_tree::WidgetTree>,
  ) -> Self {
    widget.unwrap_attr_or_else_with(|mut widget| {
      let id =
        unsafe { tree.as_mut().get_unchecked_mut() }.alloc_node(widget::PhantomWidget.box_it());
      let pointer = InnerState::new(&mut widget);
      let attr = StatefulAttr {
        id,
        state: pointer,
        tree: NonNull::from(&*tree),
      };
      (widget, attr)
    })
  }
}

impl StatefulAttr {
  pub(crate) fn from_id(id: WidgetId, mut tree: Pin<&mut widget_tree::WidgetTree>) -> Self {
    let widget = id.assert_get_mut(unsafe { tree.as_mut().get_unchecked_mut() });
    let pointer = InnerState::new(widget);
    Self {
      state: pointer,
      id,
      tree: NonNull::from(&*tree),
    }
  }
}

impl StatefulAttr {
  /// ## Safety
  /// Should ensure the pointer in attr has the same type with `W`, otherwise
  /// panic occur.
  pub unsafe fn ref_cell<W: Widget>(&self) -> StateRefCell<W> {
    StateRefCell {
      attr: self.clone(),
      type_info: PhantomData,
    }
  }
}

impl InnerState {
  fn new(widget: &mut BoxWidget) -> Self {
    Self {
      widget_ptr: Rc::new(RefCell::new((
        &*widget.widget as *const _,
        &mut *widget.widget as *mut _,
      ))),
      change_emitter: None,
    }
  }

  fn change_emitter(&mut self) -> LocalSubject<'static, (), ()> {
    self
      .change_emitter
      .get_or_insert_with(Subject::default)
      .clone()
  }

  unsafe fn borrow<W>(&self) -> Ref<W> {
    Ref::map(self.widget_ptr.borrow(), |(p, _)| &*(*p as *const W))
  }

  unsafe fn borrow_mut<W>(&mut self) -> RefMut<W> {
    RefMut::map(self.widget_ptr.borrow_mut(), |(_, p)| &mut *(*p as *mut W))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn smoke() {
    let mut tree = Box::pin(widget_tree::WidgetTree::default());
    // Simulate `Text` widget need modify its text in event callback. So return a
    // cell ref of the `Text` but not own it. Can use the `cell_ref` in closure.
    let stateful = Stateful::stateful(Text("Hello".to_string()), tree.as_mut());
    {
      stateful.ref_cell().borrow_mut().0 = "World!".to_string();
    }
    assert_eq!(&stateful.0, "World!");
  }

  #[test]
  fn inherit_from_stateful() {
    let mut render_tree = render_tree::RenderTree::default();
    let mut tree = Box::pin(widget_tree::WidgetTree::default());

    let stateful = Stateful::stateful(Text("Hello".to_string()), tree.as_mut());
    // now key widget inherit from stateful widget.
    let key = stateful.with_key(1);
    let tree = unsafe { tree.as_mut().get_unchecked_mut() };
    let id = tree.set_root(key.box_it(), &mut render_tree);

    let key_back = id
      .get(tree)
      .and_then(|w| w.downcast_attr_widget::<Key>())
      .map(|k| k.key());
    assert!(key_back.is_some());
  }

  #[test]
  fn state_notify_and_relayout() {
    use std::{cell::RefCell, rc::Rc};
    let notified_count = Rc::new(RefCell::new(0));
    let cnc = notified_count.clone();

    let mut render_tree = render_tree::RenderTree::default();
    let mut tree = Box::pin(widget_tree::WidgetTree::default());
    let mut sized_box =
      Stateful::stateful(SizedBox::empty_box(Size::new(100., 100.)), tree.as_mut());

    sized_box
      .change_stream()
      .subscribe(move |_| *cnc.borrow_mut() += 1);

    let changed_size = Rc::new(RefCell::new(Size::zero()));
    let c_changed_size = changed_size.clone();
    sized_box.pick_state(|w| w.size).subscribe(move |size| {
      *c_changed_size.borrow_mut() = size.after;
    });

    let mut state = sized_box.ref_cell();
    let tree = unsafe { tree.as_mut().get_unchecked_mut() };
    tree.set_root(sized_box.box_it(), &mut render_tree);

    {
      state.borrow_mut();
      state.borrow_mut();
    }

    assert_eq!(*notified_count.borrow(), 2);
    assert_eq!(tree.changed_widgets().len(), 1);
    assert_eq!(&*changed_size.borrow(), &Size::new(100., 100.));
  }

  #[test]
  fn fix_pin_widget_node() {
    #[derive(Debug)]
    struct TestWidget;

    impl CombinationWidget for TestWidget {
      fn build(&self, ctx: &mut BuildCtx) -> BoxWidget {
        SizedBox::empty_box(Size::new(100., 100.))
          .into_stateful(ctx)
          .box_it()
      }
    }

    impl_widget_for_combination_widget!(TestWidget);

    let mut wnd = window::Window::without_render(TestWidget.box_it(), Size::new(500., 500.));
    wnd.render_ready();
    let tree = wnd.widget_tree();
    assert_eq!(tree.count(), 2);
  }
}
