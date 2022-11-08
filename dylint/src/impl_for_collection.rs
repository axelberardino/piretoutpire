use clippy_utils::{diagnostics::span_lint, match_any_def_paths};
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::symbol::sym;

declare_lint! {
    /// **What it does:**
    /// Lint to ensure that implementation from a type T into a Collection is what you really want to do.
    ///
    /// **Why is this bad?**
    /// Implementation from a type T into a Collection should be avoided, instead you should
    /// probably implement FromIterator
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// struct Foo(T);
    ///
    /// impl From<T> for Vec<T> {
    ///     fn from(value: T) -> Vec<T> {
    ///         ...etc
    ///     }
    /// }
    /// ```
    pub IMPL_FOR_COLLECTION,
    Warn,
    "Warns on for Collections implementation."
}

declare_lint_pass!(ImplForCollection => [IMPL_FOR_COLLECTION]);

impl<'tcx, 'hir> LateLintPass<'tcx> for ImplForCollection {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        // check for `impl From<???> for ..` or `impl TryFrom<???> for ..`
        if_chain! {
        if let hir::ItemKind::Impl(kind) = &item.kind;
        if let Some(impl_trait_ref) = cx.tcx.impl_trait_ref(item.def_id);
        if cx.tcx.is_diagnostic_item(sym::From, impl_trait_ref.def_id) || cx.tcx.is_diagnostic_item(sym::TryFrom, impl_trait_ref.def_id);
        if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &kind.self_ty.kind;
        if need_to_warn(cx, path.res);
        then {
            span_lint(
                cx,
                IMPL_FOR_COLLECTION,
                item.span,
                "implementing a for collection cannot be done over T",
                );
        }
        }
    }
}

fn need_to_warn(cx: &LateContext<'_>, def: hir::def::Res) -> bool {
    if_chain! {
        if let hir::def::Res::Def(_, def_id) = def;
        if let Some(_) = match_any_def_paths(
            cx,
            def_id,
            &[
                &["alloc", "vec", "Vec"],
                &["std", "vec", "Vec"],
                &["std", "collections", "hash", "set", "HashSet"],
                &["std", "collections", "hash", "map", "HashMap"],
                &["hashbrown", "map", "HashMap"],
                &["hashbrown", "set", "HashSet"],
            ],
        );
        then {
            true
        } else {
            false
        }
    }
}
