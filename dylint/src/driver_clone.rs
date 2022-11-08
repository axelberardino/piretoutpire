use clippy_utils::{diagnostics::span_lint_and_sugg, source::snippet_with_macro_callsite};
use if_chain::if_chain;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::TyKind;
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::symbol::sym;

declare_lint! {
    /// **What it does:**
    /// Lint to prevent usage of `driver.clone()`
    ///
    /// **Why is this bad?**
    /// While `.clone()` usage on a driver is a cheap clone, it is not self explanatory.
    /// It has been raise that the use of clone everywhere is confusing and that a `Driver::clone(&driver)` like for `Arc` is more clear.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// use backend_position::driver::Driver as PositionDriver;
    ///
    /// Context {
    ///   position: PositionDriver;
    /// }
    ///
    /// impl {
    ///   fn do_something(&self) {
    ///     do_internal_thing(self.position.clone());
    ///   }
    /// }
    /// ```
    ///
    /// Use instead:
    /// ```rust
    /// impl {
    ///   fn do_something(&self) {
    ///     do_internal_thing(PositionDriver::clone(&self.position));
    ///   }
    /// }
    /// ```
    pub DRIVER_CLONE,
    Warn,
    "Warns on deriving Clone on gRPC driver"
}

declare_lint_pass!(DriverClone => [DRIVER_CLONE]);

impl<'tcx, 'hir> LateLintPass<'tcx> for DriverClone {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'_>) {
        if_chain! {
            if let hir::ExprKind::MethodCall(path, [callee], _span) = &expr.kind;
            if path.ident.name == sym::clone;
            if let ty = cx.typeck_results().expr_ty(callee);
            if let TyKind::Adt(def, _) = ty.peel_refs().kind();
            let mut path = cx.get_def_path(def.did()).into_iter().map(|sym| sym.to_ident_string()).rev().take(3);
            if matches!(path.next().as_deref(), Some("Driver"));
            if matches!(path.next().as_deref(), Some("driver"));
            if matches!(path.next().as_deref(), Some("grpc"));
            let snippet = snippet_with_macro_callsite(cx, callee.span, "..");
            then {
                let sugg = if ty.is_ref() {
                    format!("Driver::clone({})", snippet)
                } else {
                    format!("Driver::clone(&{})", snippet)
                };

                span_lint_and_sugg(
                    cx,
                    DRIVER_CLONE,
                    expr.span,
                    "using `.clone()` on a gRPC driver",
                    "try this",
                    sugg,
                    Applicability::Unspecified,
                );
            }
        }
    }
}
