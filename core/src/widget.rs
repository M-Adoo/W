#[doc(hidden)]
pub use std::{
  any::{Any, TypeId},
  marker::PhantomData,
};
pub mod key;
pub mod layout;
use algo::ShareResource;
pub use layout::*;
pub mod stateful;
pub mod text;
mod theme;
pub use theme::*;
pub(crate) mod widget_tree;
pub use crate::dynamic_widget::{ConstExprWidget, ExprWidget};
pub use crate::widget::text::Text;
pub use key::{Key, KeyWidget};
pub use stateful::*;
mod cursor;
pub use cursor::Cursor;
pub use winit::window::CursorIcon;
mod margin;
pub use margin::*;
mod padding;
pub use padding::*;
mod icon;
pub use icon::*;
mod svg;
pub use svg::*;
mod box_decoration;
pub use box_decoration::*;
mod checkbox;
pub use checkbox::*;
mod scrollable;
pub use scrollable::*;
mod path;
pub use path::*;
mod grid_view;
pub use grid_view::*;
// mod scroll_view;
// pub use scroll_view::ScrollView;
// mod scrollbar;
pub mod data_widget;
pub use data_widget::DataWidget;

mod void;
use self::widget_tree::BoxClamp;
pub use void::Void;
mod composed_widget;
pub(crate) use composed_widget::ComposedWidget;
mod lifecycle;
pub use lifecycle::*;

pub trait Compose {
  /// Describes the part of the user interface represented by this widget.
  /// Called by framework, should never directly call it.
  fn compose(this: StateWidget<Self>, ctx: &mut BuildCtx) -> Widget
  where
    Self: Sized;
}

/// RenderWidget is a widget which want to paint something or do a layout to
/// calc itself size and update children positions.
///
/// Render Widget should at least implement one of `Layout` or `Paint`, if all
/// of `as_layout` and `as_paint` return None, the widget will not display.
///
/// If `as_layout` return none, widget size will detected by its single child if
/// it has or as large as possible.
pub trait Render: Query {
  /// Do the work of computing the layout for this widget, and return the
  /// size it need.
  ///
  /// In implementing this function, You are responsible for calling every
  /// children's perform_layout across the `LayoutCtx`
  fn perform_layout(&self, clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size;

  /// `paint` is a low level trait to help you draw your widget to paint device
  /// across `PaintingCtx::painter` by itself coordinate system. Not care
  /// about children's paint in this method, framework will call children's
  /// paint individual. And framework guarantee always paint parent before
  /// children.
  fn paint(&self, ctx: &mut PaintingCtx);

  /// Whether the constraints from parent are the only input to detect the
  /// widget size, and child nodes' size not affect its size.
  fn only_sized_by_parent(&self) -> bool { false }
}

/// Enum to store both stateless and stateful widget.
pub enum StateWidget<W> {
  Stateless(W),
  Stateful(Stateful<W>),
}
pub struct Widget(pub(crate) WidgetInner);

#[marker]
pub(crate) trait WidgetMarker {}
impl<W: Compose> WidgetMarker for W {}
impl<W: ComposeSingleChild> WidgetMarker for W {}
impl<W: ComposeMultiChild> WidgetMarker for W {}
impl<W: Render> WidgetMarker for W {}

/// A trait to query dynamic type and its inner type on runtime, use this trait
/// to provide type information you want framework know.
pub trait Query {
  /// A type can composed by others, this method query all type(include self)
  /// match the type id, and call the callback one by one. The callback accept
  /// an `& dyn Any` of the target type, and return if it want to continue.
  fn query_all(
    &self,
    type_id: TypeId,
    callback: &mut dyn FnMut(&dyn Any) -> bool,
    order: QueryOrder,
  );
  // todo: remove mut access
  /// A type can composed by others, this method query all type(include self)
  /// match the type id, and call the callback one by one. The callback accept
  /// an `&mut dyn Any` of the target type, and return if want to continue.
  fn query_all_mut(
    &mut self,
    type_id: TypeId,
    callback: &mut dyn FnMut(&mut dyn Any) -> bool,
    order: QueryOrder,
  );
}

#[derive(Clone, Copy)]
pub enum QueryOrder {
  InnerFirst,
  OutsideFirst,
}

pub(crate) type BoxedSingleChild = Box<SingleChildWidget<Box<dyn Render>>>;
pub(crate) type BoxedMultiChild = MultiChildWidget<Box<dyn Render>>;

pub(crate) enum WidgetInner {
  Compose(Box<dyn FnOnce(&mut BuildCtx) -> Widget>),
  Render(Box<dyn Render>),
  SingleChild(BoxedSingleChild),
  MultiChild(BoxedMultiChild),
  Expr(ExprWidget<()>),
}

/// Trait to detect if a type is match the `type_id`.
pub trait QueryFiler {
  /// query self type by type id, and return a reference of `Any` trait to cast
  /// to target type if type match.
  fn query_filter(&self, type_id: TypeId) -> Option<&dyn Any>;
  /// query self type by type id, and return a mut reference of `Any` trait to
  /// cast to target type if type match.
  fn query_filter_mut(&mut self, type_id: TypeId) -> Option<&mut dyn Any>;
}

impl<W: 'static> QueryFiler for W {
  #[inline]
  fn query_filter(&self, type_id: TypeId) -> Option<&dyn Any> {
    (self.type_id() == type_id).then(|| self as &dyn Any)
  }

  #[inline]
  fn query_filter_mut(&mut self, type_id: TypeId) -> Option<&mut dyn Any> {
    ((&*self).type_id() == type_id).then(|| self as &mut dyn Any)
  }
}

impl<'a> dyn Render + 'a {
  #[inline]
  pub fn query_all_type<T: Any>(&self, mut callback: impl FnMut(&T) -> bool, order: QueryOrder) {
    self.query_all(
      TypeId::of::<T>(),
      &mut |a: &dyn Any| a.downcast_ref().map_or(true, |t| callback(t)),
      order,
    )
  }

  #[inline]
  pub fn query_all_type_mut<T: Any>(
    &mut self,
    mut callback: impl FnMut(&mut T) -> bool,
    order: QueryOrder,
  ) {
    self.query_all_mut(
      TypeId::of::<T>(),
      &mut |a: &mut dyn Any| a.downcast_mut().map_or(true, |t| callback(t)),
      order,
    )
  }

  /// Query the first match type in all type by special order, and call
  /// `callback`
  pub fn query_on_first_type<T: Any>(&self, order: QueryOrder, callback: impl FnOnce(&T)) {
    let mut callback = Some(callback);
    self.query_all_type(
      move |a| {
        let cb = callback.take().expect("should only call once");
        cb(a);
        false
      },
      order,
    );
  }

  /// Query the first match type in all type by special order then call
  /// `callback`.

  pub fn query_on_first_type_mut<T: Any>(
    &mut self,
    order: QueryOrder,
    callback: impl FnOnce(&mut T),
  ) {
    let mut callback = Some(callback);
    self.query_all_type_mut(
      move |a| {
        let cb = callback.take().expect("should only call once");
        cb(a);
        false
      },
      order,
    );
  }

  pub fn contain_type<T: Any>(&self, order: QueryOrder) -> bool {
    let mut hit = false;
    self.query_all_type(
      |_: &T| {
        hit = true;
        false
      },
      order,
    );
    hit
  }
}

pub trait IntoWidget<M: ?Sized> {
  fn into_widget(self) -> Widget;
}

impl IntoWidget<Widget> for Widget {
  #[inline]
  fn into_widget(self) -> Widget { self }
}

impl<C: Compose + Into<StateWidget<C>> + 'static> IntoWidget<dyn Compose> for C {
  fn into_widget(self) -> Widget {
    Widget(WidgetInner::Compose(Box::new(|ctx| {
      ComposedWidget::<Widget, C>::new(Compose::compose(self.into(), ctx)).into_widget()
    })))
  }
}

impl<R: Render + 'static> IntoWidget<dyn Render> for R {
  #[inline]
  fn into_widget(self) -> Widget { Widget(WidgetInner::Render(Box::new(self))) }
}

impl<F: FnOnce(&mut BuildCtx) -> Widget + 'static> IntoWidget<F> for F {
  #[inline]
  fn into_widget(self) -> Widget { Widget(WidgetInner::Compose(Box::new(self))) }
}

#[macro_export]
macro_rules! impl_proxy_query {
  ($field: tt) => {
    #[inline]
    fn query_all(
      &self,
      type_id: TypeId,
      callback: &mut dyn FnMut(&dyn Any) -> bool,
      order: QueryOrder,
    ) {
      self.$field.query_all(type_id, callback, order)
    }

    #[inline]
    fn query_all_mut(
      &mut self,
      type_id: TypeId,
      callback: &mut dyn FnMut(&mut dyn Any) -> bool,
      order: QueryOrder,
    ) {
      self.$field.query_all_mut(type_id, callback, order)
    }
  };
}

#[macro_export]
macro_rules! impl_query_self_only {
  () => {
    #[inline]
    fn query_all(
      &self,
      type_id: TypeId,
      callback: &mut dyn FnMut(&dyn Any) -> bool,
      _: QueryOrder,
    ) {
      if let Some(a) = self.query_filter(type_id) {
        callback(a);
      }
    }

    #[inline]
    fn query_all_mut(
      &mut self,
      type_id: TypeId,
      callback: &mut dyn FnMut(&mut dyn Any) -> bool,
      _: QueryOrder,
    ) {
      if let Some(a) = self.query_filter_mut(type_id) {
        callback(a);
      }
    }
  };
}

impl<T: Render> Render for algo::ShareResource<T> {
  #[inline]
  fn perform_layout(&self, clamp: BoxClamp, ctx: &mut LayoutCtx) -> Size {
    T::perform_layout(self, clamp, ctx)
  }

  #[inline]
  fn paint(&self, ctx: &mut PaintingCtx) { T::paint(self, ctx) }

  fn only_sized_by_parent(&self) -> bool { T::only_sized_by_parent(self) }
}

impl<T: Query> Query for ShareResource<T> {
  fn query_all(
    &self,
    type_id: TypeId,
    callback: &mut dyn FnMut(&dyn Any) -> bool,
    order: QueryOrder,
  ) {
    (&**self).query_all(type_id, callback, order)
  }

  fn query_all_mut(&mut self, _: TypeId, _: &mut dyn FnMut(&mut dyn Any) -> bool, _: QueryOrder) {
    // resource can not be queried as mut.
  }
}

impl<W> From<W> for StateWidget<W> {
  #[inline]
  fn from(w: W) -> Self { StateWidget::Stateless(w) }
}

impl<W> From<Stateful<W>> for StateWidget<W> {
  #[inline]
  fn from(w: Stateful<W>) -> Self { StateWidget::Stateful(w) }
}

impl<W: IntoStateful> StateWidget<W> {
  pub fn into_stateful(self) -> Stateful<W> {
    match self {
      StateWidget::Stateless(w) => w.into_stateful(),
      StateWidget::Stateful(w) => w,
    }
  }
}
