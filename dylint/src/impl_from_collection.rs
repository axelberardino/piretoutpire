use clippy_utils::{diagnostics::span_lint, match_any_def_paths};
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::symbol::sym;

declare_lint! {
    /// **What it does:**
    /// Lint From/TryFrom implementation over collections to ensure it's really what you want to do.
    ///
    /// **Why is this bad?**
    /// A From/TryFrom implementation over collections should be avoided
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// struct Foo(Vec<String>);
    ///
    /// impl From<Vec<T>> for T {
    ///     fn from(value: Vec<T>>) -> T {
    ///         ...etc
    ///     }
    /// }
    /// ```
    pub IMPL_FROM_COLLECTION,
    Warn,
    "Warns on Generic implementation."
}

declare_lint_pass!(ImplFromCollection => [IMPL_FROM_COLLECTION]);

impl<'tcx, 'hir> LateLintPass<'tcx> for ImplFromCollection {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        // check for `impl From<???> for ..` or `impl TryFrom<???> for ..`
        if_chain! {
            if let hir::ItemKind::Impl(kind) = &item.kind;
            if let Some(of_trait) = &kind.of_trait;
            if let Some(impl_trait_ref) = cx.tcx.impl_trait_ref(item.def_id);
            if cx.tcx.is_diagnostic_item(sym::From, impl_trait_ref.def_id) || cx.tcx.is_diagnostic_item(sym::TryFrom, impl_trait_ref.def_id);
            then {
                for segment in of_trait.path.segments {
                    if let Some(args) = segment.args {
                        for arg in args.args {
                            if_chain! {
                                if let hir::GenericArg::Type(ty) = arg;
                                if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &ty.kind;
                                if need_to_warn(cx, path.res);
                                then {
                                    span_lint(
                                        cx,
                                        IMPL_FROM_COLLECTION,
                                        item.span,
                                        "implementing TryFrom/From cannot be done over collections",
                                        );
                                }
                            }
                        }
                    }
                }
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
