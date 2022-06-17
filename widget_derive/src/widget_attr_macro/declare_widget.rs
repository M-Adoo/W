use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use std::collections::BTreeMap;
use syn::{
  bracketed,
  parse::{Parse, ParseStream},
  spanned::Spanned,
  token::{self, Brace},
  visit_mut::VisitMut,
  Expr, Ident, Path,
};
mod widget_gen;
use crate::{
  error::{DeclareError, DeclareWarning},
  widget_attr_macro::{ribir_prefix_variable, UsedType},
};
mod builtin_fields;
pub use builtin_fields::*;
pub use widget_gen::WidgetGen;

use super::{
  kw,
  widget_macro::{is_expr_keyword, IfGuard, EXPR_FIELD, EXPR_WIDGET},
  DeclareCtx, Id, NameUsed, Result, Scope, ScopeUsedInfo, UsedPart,
};

#[derive(Debug)]
pub struct DeclareWidget {
  pub path: Path,
  brace_token: Brace,
  // the name of this widget specified by `id` attr.
  pub named: Option<Id>,
  fields: Vec<DeclareField>,
  pub builtin: BuiltinFieldWidgets,
  pub children: Vec<DeclareWidget>,
}

#[derive(Clone, Debug)]
pub struct DeclareField {
  pub skip_nc: Option<SkipNcAttr>,
  pub member: Ident,
  pub if_guard: Option<IfGuard>,
  pub colon_token: Option<token::Colon>,
  pub expr: Expr,
  pub used_name_info: ScopeUsedInfo,
}

#[derive(Clone, Debug)]
pub struct SkipNcAttr {
  pound_token: token::Pound,
  bracket_token: token::Bracket,
  skip_nc_meta: kw::skip_nc,
}

macro_rules! assign_uninit_field {
  ($self: ident.$name: ident, $field: ident) => {
    assign_uninit_field!($self.$name, $field, $name)
  };
  ($left: expr, $right: ident, $name: ident) => {
    if $left.is_none() {
      $left = Some($right);
      Ok(())
    } else {
      Err(syn::Error::new(
        $right.span(),
        format!("`{}` declare more than once", stringify!($name)).as_str(),
      ))
    }
  };
}

pub(crate) use assign_uninit_field;

impl ToTokens for SkipNcAttr {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    self.pound_token.to_tokens(tokens);
    self.bracket_token.surround(tokens, |tokens| {
      self.skip_nc_meta.to_tokens(tokens);
    })
  }
}

impl ToTokens for DeclareField {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    self.member.to_tokens(tokens);
    self.colon_token.to_tokens(tokens);
    let expr = &self.expr;
    if let Some(if_guard) = self.if_guard.as_ref() {
      tokens.extend(quote! {
        #if_guard {
          #expr
        } else {
          <_>::default()
        }
      })
    } else if self.colon_token.is_some() {
      expr.to_tokens(tokens)
    }
  }
}

impl Spanned for DeclareWidget {
  fn span(&self) -> Span { self.path.span().join(self.brace_token.span).unwrap() }
}

impl Parse for DeclareWidget {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let path = input.parse()?;
    let content;
    let brace_token = syn::braced!(content in input);
    let mut named: Option<Id> = None;
    let mut fields = vec![];
    let mut builtin = BuiltinFieldWidgets::default();
    let mut children = vec![];
    loop {
      if content.is_empty() {
        break;
      }

      if content.peek(Ident) && content.peek2(token::Brace) {
        children.push(content.parse()?);
      } else {
        let is_id = content.peek(kw::id);
        let f: DeclareField = content.parse()?;
        if !children.is_empty() {
          return Err(syn::Error::new(
            f.span(),
            "Field should always declare before children.",
          ));
        }

        if is_id {
          let id = Id::from_declare_field(f)?;
          assign_uninit_field!(named, id, id)?;
        } else if let Some(ty) = FIELD_WIDGET_TYPE.get(f.member.to_string().as_str()) {
          builtin.assign_builtin_field(ty, f)?;
        } else {
          fields.push(f);
        }

        if !content.is_empty() {
          content.parse::<token::Comma>()?;
        }
      }
    }

    Ok(DeclareWidget {
      path,
      brace_token,
      named,
      fields,
      builtin,
      children,
    })
  }
}

impl Parse for SkipNcAttr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let pound_token = input.parse()?;
    let content;
    let bracket_token = bracketed!(content in input);
    Ok(Self {
      pound_token,
      bracket_token,
      skip_nc_meta: content.parse()?,
    })
  }
}

impl Parse for DeclareField {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let skip_nc = try_parse_skip_nc(input)?;
    let member: Ident = input.parse()?;
    let if_guard = if input.peek(token::If) {
      Some(input.parse()?)
    } else {
      None
    };
    let colon_token: Option<_> = if if_guard.is_some() {
      Some(input.parse()?)
    } else {
      input.parse()?
    };

    let expr = if colon_token.is_some() {
      input.parse()?
    } else {
      Expr::Path(syn::ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: Path::from(member.clone()),
      })
    };

    Ok(DeclareField {
      skip_nc,
      member,
      if_guard,
      colon_token,
      expr,
      used_name_info: ScopeUsedInfo::default(),
    })
  }
}

impl DeclareField {
  pub fn used_part(&self) -> Option<UsedPart> { self.used_name_info.user_part(Scope::Field(self)) }
}

pub fn try_parse_skip_nc(input: ParseStream) -> syn::Result<Option<SkipNcAttr>> {
  if input.peek(token::Pound) {
    Ok(Some(input.parse()?))
  } else {
    Ok(None)
  }
}

impl DeclareCtx {
  pub fn visit_declare_widget_mut(&mut self, w: &mut DeclareWidget) {
    let mut ctx = self.stack_push();
    w.fields
      .iter_mut()
      .for_each(|f| ctx.visit_declare_field_mut(f));

    ctx.visit_builtin_field_widgets(&mut w.builtin);

    w.children
      .iter_mut()
      .for_each(|c| ctx.visit_declare_widget_mut(c))
  }

  pub fn visit_declare_field_mut(&mut self, f: &mut DeclareField) {
    self.visit_ident_mut(&mut f.member);
    if let Some(if_guard) = f.if_guard.as_mut() {
      self.visit_if_guard_mut(if_guard);
    }
    self.visit_expr_mut(&mut f.expr);

    f.used_name_info = self.take_current_used_info();
  }

  pub fn visit_builtin_field_widgets(&mut self, builtin: &mut BuiltinFieldWidgets) {
    builtin.visit_builtin_fields_mut(self);
  }
}

impl DeclareWidget {
  pub fn host_and_builtin_widgets_tokens<'a>(
    &'a self,
    name: &'a Ident,
    ctx: &'a DeclareCtx,
  ) -> impl Iterator<Item = (Ident, TokenStream)> + '_ {
    let Self { path: ty, fields, .. } = self;
    let gen = WidgetGen::new(ty, name, fields);
    let host = gen.gen_widget_tokens(ctx);
    let builtin = self.builtin.widget_tokens_iter(name, ctx);
    std::iter::once((name.clone(), host)).chain(builtin)
  }

  pub fn before_generate_check(&self, ctx: &DeclareCtx) -> Result<()> {
    self.traverses_widget().try_for_each(|w| {
      if w.named.is_some() {
        w.builtin_field_if_guard_check(ctx)?;
      }
      if is_expr_keyword(&w.path) {
        if w.fields.len() != 1 || w.fields[0].member != EXPR_FIELD {
          let spans = w.fields.iter().map(|f| f.member.span().unwrap()).collect();
          return Err(DeclareError::ExprWidgetInvalidField(spans));
        }
        if let Some(guard) = w.fields[0].if_guard.as_ref() {
          return Err(DeclareError::UnsupportedIfGuard {
            name: format!("field {EXPR_FIELD} of  {EXPR_WIDGET}"),
            span: guard.span().unwrap(),
          });
        }
      }

      w.builtin.key_follow_check()
    })
  }

  pub fn warnings(&self) -> impl Iterator<Item = DeclareWarning> + '_ {
    self
      .fields
      .iter()
      .chain(self.builtin.all_builtin_fields())
      .filter(|f| self.named.is_none() || f.used_name_info.all_widgets().is_none())
      .filter_map(|f| {
        f.skip_nc
          .as_ref()
          .map(|attr| DeclareWarning::NeedlessSkipNc(attr.span().unwrap()))
      })
      .chain(self.children.iter().flat_map(|c| {
        let iter: Box<dyn Iterator<Item = DeclareWarning>> = Box::new(c.warnings());
        iter
      }))
  }

  /// return follow relationship of the named widgets,it is a key-value map,
  /// schema like
  /// ``` ascii
  /// {
  ///   widget_name: [field, {depended_widget: [position]}]
  /// }
  /// ```
  pub fn analyze_object_dependencies(&self) -> BTreeMap<Ident, NameUsed> {
    let mut follows: BTreeMap<Ident, NameUsed> = BTreeMap::new();
    self.traverses_widget().for_each(|w| {
      if let Some(name) = w.name() {
        w.builtin.collect_builtin_widget_follows(name, &mut follows);

        let w_follows: NameUsed = w.fields.iter().flat_map(|f| f.used_part()).collect();

        if !w_follows.is_empty() {
          follows.insert(name.clone(), w_follows);
        }
      }
    });

    follows
  }

  pub(crate) fn is_expr_widget(&self) -> bool {
    // if `ExprWidget` track nothing, will not as a `ExprWidget`, but use its
    // directly return value.
    is_expr_keyword(&self.path)
      && self
        .fields
        .iter()
        .any(|f| f.used_name_info.directly_used_widgets().is_some())
  }

  fn builtin_field_if_guard_check(&self, ctx: &DeclareCtx) -> Result<()> {
    let w_ref = self.name().expect("should not check anonymous widget.");
    self
      .builtin
      .all_builtin_fields()
      .filter(|f| f.if_guard.is_some())
      .try_for_each(|f| {
        let wrap_name = ribir_prefix_variable(&f.member, &w_ref.to_string());

        if ctx.is_used(&wrap_name) {
          let if_guard_span = f.if_guard.as_ref().unwrap().span().unwrap();
          let mut use_spans = vec![];
          self.traverses_widget().for_each(|w| {
            w.builtin
              .all_builtin_fields()
              .filter_map(|f| {
                f.used_name_info
                  .filter_item(|info| info.used_type.contains(UsedType::USED))
              })
              .flatten()
              .filter(|(name, _)| *name == &wrap_name)
              .for_each(|(_, info)| use_spans.extend(info.spans.iter().map(|s| s.unwrap())))
          });

          let host_span = w_ref.span().unwrap();
          let wrap_span = wrap_name.span().unwrap();
          return Err(DeclareError::DependOBuiltinFieldWithIfGuard {
            wrap_def_spans: [host_span, wrap_span, if_guard_span],
            use_spans,
            wrap_name,
          });
        }
        Ok(())
      })
  }

  pub fn traverses_widget(&self) -> impl Iterator<Item = &DeclareWidget> {
    let children: Box<dyn Iterator<Item = &DeclareWidget>> =
      Box::new(self.children.iter().flat_map(|w| w.traverses_widget()));

    std::iter::once(self).chain(children)
  }

  pub fn name(&self) -> Option<&Ident> { self.named.as_ref().map(|id| &id.name) }
}

pub fn upstream_tokens<'a>(used_widgets: impl Iterator<Item = &'a Ident> + Clone) -> TokenStream {
  let upstream = used_widgets.clone().map(|w| {
    quote_spanned! { w.span() =>  #w.change_stream() }
  });
  if used_widgets.count() > 1 {
    quote! {  observable::from_iter([#(#upstream),*]).merge_all(usize::MAX) }
  } else {
    quote! { #(#upstream)* }
  }
}
