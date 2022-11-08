use clippy_utils::{diagnostics::span_lint_and_sugg, method_calls};
use if_chain::if_chain;
use rustc_hir as hir;
use rustc_hir::{Body, Expr};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// **What it does:**
    /// Lint to prevent the build of tonic without setting the proto_builder::rerun_if_changed
    ///
    /// **Why is this bad?**
    /// When changing the proto files if you don't force a rerun you could have some issues on the CI
    ///
    /// **Known problems:** None.
    ///
    /// **Example:**
    /// ```rust
    /// use std::{
    ///      env,
    ///      path::{Path, PathBuf},
    ///  };
    ///
    ///  fn main() {
    ///      let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR env variable"));
    ///      let proto_dir = Path::new("../../../platform/libraries/proto/proto");
    ///      let protodefs = proto_dir.join("github.com/znly/protodefs");
    ///
    ///      proto_builder::rerun_if_changed(protodefs).expect("could'nt walk through directories");
    ///
    ///      tonic_build::configure()
    ///          .build_server(true)
    ///          .build_client(true)
    ///          .file_descriptor_set_path(out_dir.join("service_descriptor.bin"))
    ///          .compile(...)
    ///          .expect("unable to compile service");
    ///  }
    ///
    /// ```
    pub BUILD_RERUN_PROTO,
    Warn,
    "Warns on using tonic_build without setting the rerun_if_changed"
}

declare_lint_pass!(BuildRerunProto => [BUILD_RERUN_PROTO]);

impl<'tcx> LateLintPass<'tcx> for BuildRerunProto {
    fn check_body(&mut self, cx: &LateContext<'tcx>, body: &'tcx Body<'tcx>) {
        if let hir::ExprKind::Block(
            hir::Block {
                stmts,
                expr: _,
                hir_id: _,
                rules: _,
                span: _,
                targeted_by_break: _,
            },
            _,
        ) = &body.value.kind
        {
            let exprs: Vec<&Expr> = stmts
                .into_iter()
                .filter_map(|stmt| {
                    if let hir::StmtKind::Semi(kind) = stmt.kind {
                        Some(kind)
                    } else {
                        None
                    }
                })
                .collect();

            let mut rerun_enabled = false;
            for expr in exprs {
                if_chain! {
                    // fixed this to 20 (which allows to have 20 methods call on tonic_build)
                    let (_, filtered_exprs, _) = method_calls(expr, 20);
                    if let Some(kind) = filtered_exprs.last();
                    if let Some(filtered_expr) = kind.first();
                    if let hir::ExprKind::Call(call_expr, _) = &filtered_expr.kind;
                    if let hir::ExprKind::Path(qpath) = &call_expr.kind;
                    if let hir::QPath::Resolved(_, path) = *qpath;
                    if let Some(name) = path
                        .segments
                        .first()
                        .map(|segment| segment.ident.as_str().to_string());
                    then {
                        if name == "tonic_build" && !rerun_enabled {
                            span_lint_and_sugg(
                                cx,
                                BUILD_RERUN_PROTO,
                                expr.span,
                                "proto_builder::rerun_if_changed is not set",
                                "configure proto_builder::rerun_if_changed when using",
                                name,
                                rustc_errors::Applicability::Unspecified,
                            );
                        } else if name == "proto_builder" {
                            rerun_enabled = true;
                        }
                    }
                }
            }
        }
    }
}
