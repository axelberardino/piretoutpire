use clippy_utils::diagnostics::span_lint_and_sugg;
use hir::{ImplItem, ImplItemKind, PatKind};
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{symbol::sym, Span};

declare_lint! {
    /// **What it does:**
    /// Searches for implementations of the `From<..>` and `TryFrom<..> trait and check if the variable's name is `value` or suggests to name it `value` instead.
    ///
    /// **Why is this bad?**
    /// It's not bad to do it otherwise but it's a convention.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// struct Duration(String);
    ///
    /// impl TryFrom<Duration> for String {
    ///     type Error = AnyError;
    ///
    ///     fn try_from(duration: Duration) -> Result<Self, Self::Error> {
    ///       unreachable!()
    ///     }
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// struct Duration(String);
    ///
    /// impl TryFrom<Duration> for String {
    ///     type Error = AnyError;
    ///
    ///     fn try_from(value: Duration) -> Result<Self, Self::Error> {
    ///       unreachable!()
    ///     }
    /// }
    /// ```
    pub FROM_VARIABLE_VALUE,
    Warn,
    "Warns on implementations of `TryFrom<..>` and `From<..>` when variable is not `value`"
}

declare_lint_pass!(FromVariableValue => [FROM_VARIABLE_VALUE]);

impl<'tcx, 'hir> LateLintPass<'tcx> for FromVariableValue {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        // check for `impl From<???> for ..` or `impl TryFrom<???> for ..`
        if_chain! {
            if let hir::ItemKind::Impl(impl_) = &item.kind;
            if let Some(impl_trait_ref) = cx.tcx.impl_trait_ref(item.def_id);
            if cx.tcx.is_diagnostic_item(sym::From, impl_trait_ref.def_id) | cx.tcx.is_diagnostic_item(sym::TryFrom, impl_trait_ref.def_id);
            then {
                lint_impl_variable(cx, item.span, impl_.items);
            }
        }
    }
}

// Check if first variable name is `value`.
fn lint_impl_variable<'tcx>(cx: &LateContext<'tcx>, _impl_span: Span, impl_items: &[hir::ImplItemRef]) {
    for impl_item in impl_items {
        if_chain! {
            if (impl_item.ident.name == sym::from) | (impl_item.ident.name == sym::try_from);
            if let ImplItem { .. } = cx.tcx.hir().impl_item(impl_item.id);
            if let ImplItemKind::Fn(_, body_id) =
                cx.tcx.hir().impl_item(impl_item.id).kind;
            then {
                let body = cx.tcx.hir().body(body_id);
                for arg in body.params {
                    if let PatKind::Binding(_, _, ident, None) = arg.pat.kind {
                        let arg_name = ident.to_string();

                        if arg_name != "value" && arg_name != "_value" {
                            span_lint_and_sugg(
                                cx,
                                FROM_VARIABLE_VALUE,
                                arg.span,
                                "using something else than value is not recommended",
                                "change this variable to",
                                "value".to_owned(),
                                rustc_errors::Applicability::Unspecified,
                            );
                        }
                    }
                }

            }
        }
    }
}
