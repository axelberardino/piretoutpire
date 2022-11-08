use clippy_utils::{diagnostics::span_lint, match_any_def_paths};
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::symbol::sym;

declare_lint! {
    /// **What it does:**
    /// Lint TryFrom implementation to prevent Error type to be a tonic::Status.
    ///
    /// **Why is this bad?**
    /// We should never use a tonic::Status Error type for TryFrom implementation.
    /// It should be an AnyError or a typed error.
    /// When you TryFrom your struct `T` into `AnyResult<U>` and you are in a "grpc service" context
    /// you should map_err your AnyError/TypedError into a tonic::Status rather than returning a tonic::Status.
    /// This will provide the possibility to reuse this impl for other context than gprc's one.
    /// When doing a conversion type, you should not introduce an hard dependency on something very specific.
    /// Implementing a tonic::status error type lead your crate to depend closely to tonic. But your crate should remains generic, and not force the user to use any protocol.
    /// Instead, it should be a generic AnyError or a typed error. It's the role of the caller to handle grpc error on top of your generic error.
    /// This will provide the possibility to reuse this impl for other context than gprc's one and remove a dependency that might not be necessary.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// impl TryFrom<T> for U {
    ///     type Error = tonic::Status;
    ///
    ///     fn try_from(value: T) -> Result<Self, Self::Error> {
    ///            value
    ///                .parse::<U>()
    ///                .map_err(|_err| tonic::Status::invalid_argument("invalid T"))
    ///                .map(Into::into)
    ///     }
    /// }
    /// ```
    ///
    /// Use instead:
    /// ```rust
    /// impl TryFrom<T> for U {
    ///     type Error = AnyError;
    ///
    ///     fn try_from(value: T) -> Result<Self, Self::Error> {
    ///            value
    ///                .parse::<U>()
    ///                .map(Into::into)
    ///     }
    /// }
    /// ```
    pub TRY_FROM_TONIC_ERROR,
    Warn,
    "Warns on TryFrom implementation returning a tonic::Status."
}

declare_lint_pass!(TryFromTonicError => [TRY_FROM_TONIC_ERROR]);

impl<'tcx, 'hir> LateLintPass<'tcx> for TryFromTonicError {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        if_chain! {
            if let hir::ItemKind::Impl(impl_item) = &item.kind;
            if let Some(impl_trait_ref) = cx.tcx.impl_trait_ref(item.def_id);
            if cx.tcx.is_diagnostic_item(sym::TryFrom, impl_trait_ref.def_id);
            if let Some(item) = impl_item.items.iter().filter_map(|item| (item.ident.name.as_str() == "Error").then(|| item)).next();
            if let Some(hir::Node::ImplItem(impl_item)) = cx.tcx.hir().find_by_def_id(item.id.def_id);
            if let hir::ImplItemKind::TyAlias(ty) = impl_item.kind;
            if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = ty.kind;
            if need_to_warn(cx, path.res);
            then {
                span_lint(
                    cx,
                    TRY_FROM_TONIC_ERROR,
                    item.span,
                    "TryFrom implementation should never return a tonic::Status, instead you should return AnyError or a typed error. You should map_err your AnyError/TypedError after the TryFrom.",
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
                &["tonic", "status", "Status"]
            ],
        );
        then {
            true
        } else {
            false
        }
    }
}
