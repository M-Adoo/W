pub mod icons {
  use ribir_core::{define_icon_ident, prelude::IconIdent};

  pub const BEGIN: IconIdent = IconIdent::new(0);
  define_icon_ident!(
    BEGIN,
    CHECKED,
    UNCHECKED,
    INDETERMINATE,
    ADD_CIRCLE,
    ADD,
    ARROW_BACK,
    ARROW_DROP_DOWN,
    ARROW_FORWARD,
    CANCEL,
    CHECK_BOX,
    CHECK_BOX_OUTLINE_BLANK,
    CHECK_CIRCLE,
    CHECK,
    CHEVRON_RIGHT,
    CLOSE,
    DELETE,
    DONE,
    EXPAND_MORE,
    FAVORITE,
    FILE_DOWNLOAD,
    GRADE,
    HOME,
    INDETERMINATE_CHECK_BOX,
    LOGIN,
    LOGOUT,
    MENU,
    MORE_VERT,
    REFRESH,
    SEARCH,
    SETTINGS,
    STAR,
    THEME_EXTEND
  );
}

pub mod cs {
  use ribir_core::{define_compose_style_ident, prelude::ComposeStyleIdent};
  pub const BEGIN: ComposeStyleIdent = ComposeStyleIdent::new(0);

  define_compose_style_ident! {
    BEGIN,
    SCROLLBAR_TRACK,
    SCROLLBAR_THUMB,
    H_SCROLLBAR_TRACK,
    H_SCROLLBAR_THUMB,
    V_SCROLLBAR_TRACK,
    V_SCROLLBAR_THUMB,
    INK_BAR,
    THEME_EXTEND
  }
}
