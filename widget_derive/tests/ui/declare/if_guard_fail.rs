use ribir::prelude::*;


fn if_guard_require_declare_default() {
  let guard_false = widget! {
    SizedBox {
      size if false => : Size::new(100., 100.)
    }
  };
  let (rect, _) = widget_and_its_children_box_rect(guard_false, Size::new(1000., 1000.));
}


#[widget]
fn normal_if_guard_pass(_this: (), ctx: &mut BuildCtx) {
  let guard = Some(1);
  widget! {
    declare SizedBox {
      // if guard in widget's field
      size if true => : Size::zero(),
      // if guard in data atribute
      cursor if true => : CursorIcon::Hand,
      // if guard in listener attibute
      on_tap if let Some(_) = guard  => : |_| {},
      // if guard in wrap widget.
      margin if true => : EdgeInsets::all(1.)
    }
  };
}

#[widget]
fn id_if_guard_fail(_this: (), ctx: &mut BuildCtx) {
  let guard = Some(1);
  widget! {
    declare SizedBox {
      id if true => : test,
      // if guard in widget's field
      size if true => : Size::zero(),

    }
  };
}

#[widget]
fn depend_id_behind_if_guard_fail(_this: (), ctx: &mut BuildCtx) {
  widget! {
    declare SizedBox {
      id: a,
      size: Size::zero(),
      margin if true =>:  EdgeInsets::all(0.),

      SizedBox{
        size: Size::zero(),
        margin: a.margin
      }
    }
  };
}

fn main() {}
