use clippy_utils::diagnostics::span_lint;
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::symbol::sym;

declare_lint! {
    /// **What it does:**
    /// Lint Deref implementation to ensure it's really what you want to do.
    ///
    /// **Why is this bad?**
    /// A Deref implementation is not bad, but should be managed properly and implemented only for
    /// pointer-like struct.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// struct Duration(String);
    ///
    /// impl Deref for Duration {
    ///     type Target = String;
    ///
    ///     fn deref(&self) -> &String {
    ///         &self.0
    ///     }
    /// }
    /// ```
    pub IMPL_DEREF,
    Warn,
    "Warns on Deref implementation."
}

declare_lint_pass!(ImplDeref => [IMPL_DEREF]);

impl<'tcx, 'hir> LateLintPass<'tcx> for ImplDeref {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        // check for `impl From<???> for ..` or `impl TryFrom<???> for ..`
        if_chain! {
            if let hir::ItemKind::Impl(_) = &item.kind;
            if let Some(impl_trait_ref) = cx.tcx.impl_trait_ref(item.def_id);
            if cx.tcx.is_diagnostic_item(sym::Deref, impl_trait_ref.def_id);
            then {
                span_lint(
                    cx,
                    IMPL_DEREF,
                    item.span,
                    "implementing Deref is not easy, be sure to be in a pointer-like struct",
                    );
            }
        }
    }
}
