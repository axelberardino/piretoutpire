use clippy_utils::{diagnostics::span_lint_and_sugg, match_def_path};
use if_chain::if_chain;
use rustc_hir::{Expr, ExprKind, QPath};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::{sym, Symbol};

declare_lint! {
    /// **What it does:**
    /// Lint TryFromInfallible suggests to use no fallible function instead.
    /// **Why is this bad?**
    ///
    /// Decrease the comprehension of the code.
    /// Can lead to a the implementation of a `TryFrom` due to the misused of one conversion.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// struct Unit(());
    ///
    /// impl From<()> for Unit {
    ///     fn from(value: ()) -> Self {
    ///         Self(value)
    ///     }
    /// }
    ///
    /// let _ignored: Unit = ().try_into().expect("");
    /// ```
    ///
    /// Use instead:
    /// ```rust
    /// let _ignored: Unit = ().into();
    /// ```
    pub TRY_FROM_INFALLIBLE,
    Warn,
    "Warns about using Try{From,Into} when the returned error is Infallible."
}

declare_lint_pass!(TryFromInfallible => [TRY_FROM_INFALLIBLE]);

fn check_expr<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
    let (paths, span) = match expr.kind {
        ExprKind::MethodCall(path, exprs, span) => {
            for expr in exprs {
                check_expr(cx, expr);
            }
            (vec![path], span)
        }
        ExprKind::Call(
            Expr {
                kind: ExprKind::Path(QPath::TypeRelative(_, path)),
                ..
            },
            _,
        ) => (vec![path.clone()], expr.span),
        ExprKind::Path(QPath::Resolved(_, path)) => (path.segments.iter().collect(), path.span),
        _ => {
            return;
        }
    };

    for path in paths {
        if path.ident.name == Symbol::intern("try_from") || path.ident.name == Symbol::intern("try_into") {
            span_lint_and_sugg(
                cx,
                TRY_FROM_INFALLIBLE,
                span,
                "calling a try_{into, from} with Infallible Error",
                "use the no faillable method instead",
                "from,into".to_string(),
                rustc_errors::Applicability::Unspecified,
            );
        }
    }
}

impl<'tcx, 'hir> LateLintPass<'tcx> for TryFromInfallible {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        let typeck = cx.typeck_results();

        if_chain! {
            if let rustc_middle::ty::Adt(adt, substs) = typeck.expr_ty(expr).kind();
            if cx.tcx.is_diagnostic_item(sym::Result, adt.did());
            if let rustc_middle::ty::Adt(adt, _) = substs.type_at(1).kind();
            if match_def_path(cx, adt.did(), &["core", "convert", "Infallible"]);
            then {
                check_expr(cx, expr);
            }
        }
    }
}
