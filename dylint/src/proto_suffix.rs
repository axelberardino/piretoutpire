use clippy_utils::{diagnostics::span_lint_and_sugg, get_trait_def_id, ty::implements_trait};
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_hir::Expr;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// **What it does:**
    /// Lint ProtoSuffix implementation ensures that the objects contain in proto are suffixed with `Proto`
    ///
    /// **Why is this bad?**
    /// A raw name is not bad, but it can be confusing when you read the code.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// use proto::MyType;
    ///
    /// mod proto {
    ///     pub struct MyType;
    /// }
    ///
    /// ```
    pub PROTO_SUFFIX,
    Warn,
    "Warns on proto message imported without the `Proto` suffix."
}

declare_lint_pass!(ProtoSuffix => [PROTO_SUFFIX]);

impl<'tcx> LateLintPass<'tcx> for ProtoSuffix {
    fn check_ty(&mut self, cx: &LateContext<'tcx>, ty: &'tcx hir::Ty<'tcx>) {
        if let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &ty.kind {
            if_chain! {
                if let (name, true) = need_to_warn(cx, path.segments.last().map(|segment| segment.ident.as_str().to_string()), path.res);
                then {
                    span_lint_and_sugg(
                        cx,
                        PROTO_SUFFIX,
                        path.span,
                        "using raw name is not recommended",
                        "change this variable to",
                        name,
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
                            if let (name, true) = need_to_warn(cx, path.segments.last().map(|segment| segment.ident.as_str().to_string()), path.res);
                            then {
                                span_lint_and_sugg(
                                    cx,
                                    PROTO_SUFFIX,
                                    path.span,
                                    "using raw name is not recommended",
                                    "change this variable to",
                                    name,
                                    rustc_errors::Applicability::Unspecified,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if_chain! {
            if let hir::ExprKind::Struct(hir::QPath::Resolved(_, path), _, _) = expr.kind;
            if let (name, true) = need_to_warn(cx, path.segments.last().map(|segment| segment.ident.as_str().to_string()), path.res);
            then {
                span_lint_and_sugg(
                    cx,
                    PROTO_SUFFIX,
                    path.span,
                    "using raw name is not recommended",
                    "change this variable to",
                    name,
                    rustc_errors::Applicability::Unspecified,
                );
            }
        }
    }

    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if_chain! {
            if let hir::ItemKind::Use(path, _) = item.kind;
            if let (name, true) = need_to_warn(cx, Some(item.ident.name.as_str().to_string()), path.res);
            then {
                span_lint_and_sugg(
                    cx,
                    PROTO_SUFFIX,
                    item.span,
                    "using raw name is not recommended",
                    "change this variable to",
                    name,
                    rustc_errors::Applicability::Unspecified,
                );
            }
        }
    }
}

fn need_to_warn(cx: &LateContext<'_>, name: Option<String>, def: hir::def::Res) -> (String, bool) {
    if_chain! {
        if let hir::def::Res::Def(hir::def::DefKind::Struct | hir::def::DefKind::Union | hir::def::DefKind::Enum, def_id) = def;
        if let Some(trait_def_id) = get_trait_def_id(cx, &["prost", "Message"]);
        let ty = cx.tcx.type_of(def_id);
        if let Some(name) = name;
        if name != "Bytes";
        if name != "String";
        if implements_trait(cx, ty, trait_def_id, &[]);
        let ends_with_proto = name.ends_with("Proto");
        if !ends_with_proto;
        then {
            (format!("{}Proto", name), true)
        } else {
            ("".to_owned(), false)
        }
    }
}
