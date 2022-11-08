use clippy_utils::diagnostics::span_lint;
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_hir::{Impl, Path, TraitRef};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// **What it does:**
    /// Lint to prevent from the manual implementation of Iterator/Stream.
    ///
    /// **Why is this bad?**
    /// There is no need to add extra code/complexity, most of the time from_iter does the job
    ///
    /// **Known problems:** None.
    /// ```
    pub IMPL_ITERATOR_STREAM,
    Warn,
    "Warns on manual implementation of Iterator/Stream."
}

declare_lint_pass!(ImplIteratorStream => [IMPL_ITERATOR_STREAM]);

impl<'tcx, 'hir> LateLintPass<'tcx> for ImplIteratorStream {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        if_chain! {
        if let hir::ItemKind::Impl(Impl {of_trait: Some(TraitRef { path: Path {segments, .. }, ..}), ..}) = &item.kind;
        then {
            let mut iter = segments.into_iter().map(|segment| segment.ident.name.as_str()).take(3);
            match (iter.next(), iter.next(), iter.next()) {
                (Some("Iterator"), None, None) | (Some("futures"), Some("stream"), Some("Stream")) => {
                    span_lint(
                        cx,
                        IMPL_ITERATOR_STREAM,
                        item.span,
                        "do not impl Iterator or Stream, use helpers like ::iter()",
                    );
                }
                _ => {}
            }
        }
        }
    }
}
