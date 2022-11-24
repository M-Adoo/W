use crate::prelude::*;
use ribir_core::prelude::*;

#[derive(Default, Declare)]
pub struct Tabs {
  #[declare(default = 0)]
  pub cur_idx: usize,
}

#[derive(Declare, Debug)]
pub struct InkBarStyle;

impl ComposeStyle for InkBarStyle {
  type Host = Widget;
  #[inline]
  fn compose_style(_: Stateful<Self>, host: Widget) -> Widget { host }
}

#[derive(Template)]
pub struct Tab {
  header: WidgetOf<TabHeader>,
  pane: WidgetOf<TabPane>,
}

#[derive(Declare, SingleChild)]
pub struct TabPane;

#[derive(Declare, SingleChild)]
pub struct TabHeader;

impl ComposeChild for Tabs {
  type Child = Vec<Tab>;

  fn compose_child(this: StateWidget<Self>, children: Self::Child) -> Widget {
    let mut headers = vec![];
    let mut panes = vec![];

    for tab in children.into_iter() {
      let Tab { header, pane } = tab;
      headers.push(header.child);
      panes.push(pane.child);
    }

    let tab_size = panes.len();

    widget! {
      track {
        this: this.into_stateful()
      }

      Column {
        LayoutBox {
          id: stack,
          Stack {
            Row {
              border: Border::only_bottom(BorderSide {
                width: 1., color: Palette::of(ctx).primary()
              }),
              DynWidget {
                dyns: {
                  headers.into_iter()
                    .enumerate()
                    .map(move |(idx, header)| {
                      widget! {
                        Expanded {
                          flex: 1.,
                          tap: move |_| {
                            if this.cur_idx != idx {
                              this.cur_idx = idx;
                            }
                          },
                          DynWidget {
                            h_align: HAlign::Center,
                            v_align: VAlign::Center,

                            dyns: header
                          }
                        }
                      }
                    })
                }
              }
            }
            InkBarStyle {
              id: ink_bar,
              left_anchor: 0.,
              top_anchor: 0.,
              Container {
                id: ink_box,
                size: Size::new(0., 0.),
              }
            }
          }
        }

        DynWidget {
          dyns: panes.into_iter()
            .enumerate()
            .map(move |(idx, pane)| {
              widget! {
                DynWidget {
                  visible: this.cur_idx == idx,
                  dyns: pane
                }
              }
            })
        }

      }

      on this.cur_idx {
        change: move |(_, after)| {
          let width = stack.layout_width();
          let height = stack.layout_height();
          let pos = (after as f32) * width / (tab_size as f32);
          ink_bar.left_anchor = PositionUnit::Pixel(pos);
          ink_bar.top_anchor = PositionUnit::Pixel(height - 2.);
        }
      }

      on stack.layout_width() {
        change: move |(_, after)| {
          let width = after / (tab_size as f32);
          let height = 2.;
          ink_box.size = Size::new(width, height);

          let pos = (this.cur_idx as f32) * width / (tab_size as f32);
          ink_bar.left_anchor = PositionUnit::Pixel(pos);
        }
      }

      on stack.layout_height() {
        change: move |(_, after)| {
          ink_bar.top_anchor = PositionUnit::Pixel(after - 2.);
        }
      }

      change_on ink_bar.left_anchor Animate {
        transition: transitions::EASE_IN.of(ctx),
        lerp_fn: PositionUnit::lerp_fn(ink_bar.layout_width()),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn compose_tabs() {
    widget! {
      Tabs {
        Tab {
          TabHeader {
            Void {}
          }
          TabPane {
            Void {}
          }
        }
      }
    };
  }
}
