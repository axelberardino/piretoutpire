use clippy_utils::diagnostics::span_lint_and_help;
use hir::{ExprKind, ItemKind, PatKind, VariantData};
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::symbol::Ident;

/// To allow a one-character variable, add it to the allowlist.
const VARIABLES_ALLOWLIST: [char; 6] = ['_', 'i', 'j', 'k', 'x', 'y'];

declare_lint! {
    /// **What it does:**
    /// Lint one-character variables and suggest to change it to a more understandable name.
    ///
    /// **Why is this bad?**
    /// It's better to avoid having a one-character variable and have instead a more understandable
    /// variable. It's a convention.
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// fn print(a: i32) {
    ///     println!("{}", a);
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// fn print(value: i32) {
    ///     println!("{}", value);
    /// }
    /// ```
    pub ONE_LETTER_VARIABLE,
    Warn,
    "Warns on one-character variables and suggest to use a more meaningful name."
}

declare_lint_pass!(OneLetterVariable => [ONE_LETTER_VARIABLE]);

impl<'tcx, 'hir> LateLintPass<'tcx> for OneLetterVariable {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        match &item.kind {
            ItemKind::Enum(enumdef, _) => {
                lint_ident(cx, &item.ident);
                for variant in enumdef.variants {
                    lint_ident(cx, &variant.ident);
                }
            }
            ItemKind::Struct(variantdata, _) => {
                lint_ident(cx, &item.ident);
                match variantdata {
                    VariantData::Struct(fields, _) => {
                        for field in *fields {
                            lint_ident(cx, &field.ident);
                        }
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn check_local(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Local<'tcx>) {
        lint_pat(cx, item.pat)
    }

    fn check_arm(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Arm<'tcx>) {
        lint_pat(cx, item.pat)
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Expr<'tcx>) {
        match item.kind {
            ExprKind::Let(let_expr) => {
                lint_pat(cx, let_expr.pat);
            }
            _ => {}
        }
    }
}

fn lint_ident<'tcx>(cx: &LateContext<'tcx>, ident: &Ident) {
    let param_name = ident.to_string();

    if param_name.len() == 1 && !VARIABLES_ALLOWLIST.contains(&param_name.chars().next().expect("Can't fail"))
    {
        span_lint_and_help(
            cx,
            ONE_LETTER_VARIABLE,
            ident.span,
            "single letter identifier",
            None,
            "consider using a more meaningful name",
        );
    }
}

fn lint_pat<'tcx>(cx: &LateContext<'tcx>, pat: &'tcx hir::Pat<'_>) {
    match pat.kind {
        PatKind::Binding(_, _, ident, None) => {
            lint_ident(cx, &ident);
        }
        PatKind::Struct(_, patfields, _) => {
            for patfield in patfields {
                lint_pat(cx, patfield.pat);
            }
        }
        PatKind::TupleStruct(_, pats, _) => {
            for pat in pats {
                lint_pat(cx, pat);
            }
        }
        PatKind::Tuple(subpats, _) => {
            for subpat in subpats {
                lint_pat(cx, subpat);
            }
        }
        _ => {}
    }
}
