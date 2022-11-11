#![feature(rustc_private)]
#![allow(unused_extern_crates)]

dylint_linting::dylint_library!();

extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_index;
extern crate rustc_infer;
extern crate rustc_lexer;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_parse;
extern crate rustc_parse_format;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_trait_selection;
extern crate rustc_typeck;

mod build_rerun_proto;
mod cql_statement;
mod driver_clone;
mod from_variable_value;
mod impl_deref;
mod impl_for_collection;
mod impl_from_collection;
mod impl_iterator_stream;
mod one_letter_variable;
mod proto_suffix;
mod try_from_infallible;
mod try_from_tonic_error;

#[doc(hidden)]
#[no_mangle]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[
        build_rerun_proto::BUILD_RERUN_PROTO,
        cql_statement::CQL_STATEMENT,
        driver_clone::DRIVER_CLONE,
        from_variable_value::FROM_VARIABLE_VALUE,
        impl_deref::IMPL_DEREF,
        // impl_for_collection::IMPL_FOR_COLLECTION,
        impl_from_collection::IMPL_FROM_COLLECTION,
        impl_iterator_stream::IMPL_ITERATOR_STREAM,
        one_letter_variable::ONE_LETTER_VARIABLE,
        proto_suffix::PROTO_SUFFIX,
        try_from_infallible::TRY_FROM_INFALLIBLE,
        try_from_tonic_error::TRY_FROM_TONIC_ERROR,
    ]);

    lint_store.register_late_pass(|| Box::new(build_rerun_proto::BuildRerunProto));
    lint_store.register_late_pass(|| Box::new(cql_statement::CqlStatement));
    lint_store.register_late_pass(|| Box::new(driver_clone::DriverClone));
    lint_store.register_late_pass(|| Box::new(from_variable_value::FromVariableValue));
    lint_store.register_late_pass(|| Box::new(impl_deref::ImplDeref));
    // lint_store.register_late_pass(|| Box::new(impl_for_collection::ImplForCollection));
    lint_store.register_late_pass(|| Box::new(impl_from_collection::ImplFromCollection));
    lint_store.register_late_pass(|| Box::new(impl_iterator_stream::ImplIteratorStream));
    lint_store.register_late_pass(|| Box::new(one_letter_variable::OneLetterVariable));
    lint_store.register_late_pass(|| Box::new(proto_suffix::ProtoSuffix));
    lint_store.register_late_pass(|| Box::new(try_from_infallible::TryFromInfallible));
    lint_store.register_late_pass(|| Box::new(try_from_tonic_error::TryFromTonicError));
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
