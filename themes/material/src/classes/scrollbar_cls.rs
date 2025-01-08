use ribir_core::prelude::*;
use ribir_widgets::prelude::*;

use crate::md;

const THUMB_MIN_SIZE: f32 = 12.;

pub(super) fn init(classes: &mut Classes) {
  // In this theme, the scrollbar is positioned floating on the scroll content
  // widget, so there is no need for any additional padding or adjustments to the
  // content widget.
  //
  // However, we also provide an empty class implementation to prevent it from
  // inheriting the ancestor class implementation of `SCROLL_CLIENT_AREA`.
  classes.insert(SCROLL_CLIENT_AREA, empty_cls);

  classes.insert(H_SCROLL_THUMB, style_class! {
    background: BuildCtx::color(),
    border_radius: md::RADIUS_4,
    margin: EdgeInsets::vertical(1.),
    clamp: BoxClamp::min_width(THUMB_MIN_SIZE).with_fixed_height(md::THICKNESS_8)
  });
  classes.insert(V_SCROLL_THUMB, style_class! {
    background: BuildCtx::color(),
    border_radius: md::RADIUS_4,
    margin: EdgeInsets::horizontal(1.),
    clamp: BoxClamp::min_height(THUMB_MIN_SIZE).with_fixed_width(md::THICKNESS_8)
  });

  classes.insert(H_SCROLL_TRACK, |w| style_track(w, true));
  classes.insert(V_SCROLL_TRACK, |w| style_track(w, false));
}

fn style_track(w: Widget, is_hor: bool) -> Widget {
  rdl! {
    let scroll = Provider::of::<Stateful<ScrollableWidget>>(BuildCtx::get()).unwrap();
    let mut w = FatObj::new(w);
    if is_hor {
      w = w.v_align(VAlign::Bottom);
    } else {
      w = w.h_align(HAlign::Right);
    }
    let mut w = @ $w {
      opacity: 0.,
      visible: false,
      background: {
        let color = BuildCtx::container_color();
        pipe!(if $w.is_hover() { color } else { color.with_alpha(0.)})
      },
    };

    let trans = EasingTransition {
      easing: md::easing::STANDARD,
      duration: md::easing::duration::MEDIUM2
    };
    // Smoothly fade in and out the scrollbar.
    part_writer!(&mut w.opacity).transition(trans.clone());
    // Smoothly display the background.
    part_writer!(&mut w.background).transition(trans);

    // Show the scrollbar when scrolling.
    let mut fade: Option<TaskHandle<_>> = None;
    let auto_hide = move |_| {
      $w.write().opacity = 1.;
      $w.write().visible = true;
      if let Some(f) = fade.take() {
        f.unsubscribe();
      }
      let u = observable::timer((), Duration::from_secs(3), AppCtx::scheduler())
        .filter(move |_| !$w.is_hover())
        .subscribe(move |_| {
          $w.write().opacity = 0.;
          $w.write().visible = false;
        });
      fade = Some(u);
    };

    let u = if is_hor {
      watch!(($scroll).get_scroll_pos().x)
        .distinct_until_changed()
        .subscribe(auto_hide)
    } else {
      watch!(($scroll).get_scroll_pos().y)
        .distinct_until_changed()
        .subscribe(auto_hide)
    };

    @ $w { on_disposed: move |_| u.unsubscribe() }
  }
  .into_widget()
}
