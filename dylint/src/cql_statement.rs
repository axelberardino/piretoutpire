use clippy_utils::{diagnostics::span_lint_and_sugg, match_def_path};
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// **What it does:**
    /// Lint CQLStatement implementation ensures that code making calls to Scylla are using the CQL Observer
    ///
    /// **Why is this bad?**
    /// If we don't use the CQL Observer we can't track metrics of the calls made to Scylla and therefore we are
    /// on the fog regarding what's happening.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// use scylla::statement::prepared_statement::PreparedStatement;
    ///
    /// pub struct MyStruct {
    ///     query: PreparedStatement,
    /// }
    ///
    /// ```
    pub CQL_STATEMENT,
    Warn,
    "Warns on CQL calls wihtout using CQL Observer."
}

declare_lint_pass!(CqlStatement => [CQL_STATEMENT]);

impl<'tcx> LateLintPass<'tcx> for CqlStatement {
    fn check_ty(&mut self, cx: &LateContext<'tcx>, ty: &'tcx hir::Ty<'tcx>) {
        if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &ty.kind {
            if_chain! {
                if need_to_warn(cx, path.res);
                then {
                    span_lint_and_sugg(
                        cx,
                        CQL_STATEMENT,
                        path.span,
                        "calling scylla directly is not recommended",
                        "consider using CQL Observer",
                        "drivers::scylladb::QueryStatement".to_owned(),
                        rustc_errors::Applicability::Unspecified,
                    );
                }
            }
            for segment in path.segments {
                if let Some(args) = segment.args {
                    for arg in args.args {
                        if_chain! {
                            if let hir::GenericArg::Type(ty) = arg;
                            if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &ty.kind;
                            if need_to_warn(cx, path.res);
                            then {
                                span_lint_and_sugg(
                                    cx,
                                    CQL_STATEMENT,
                                    path.span,
                                    "calling scylla directly is not recommended",
                                    "consider using CQL Observer",
                                    "drivers::scylladb::QueryStatement".to_owned(),
                                    rustc_errors::Applicability::Unspecified,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn need_to_warn(cx: &LateContext<'_>, def: hir::def::Res) -> bool {
    if let hir::def::Res::Def(hir::def::DefKind::Struct, def_id) = def {
        return match_def_path(
            cx,
            def_id,
            &["scylla", "statement", "prepared_statement", "PreparedStatement"],
        );
    }
    false
}
