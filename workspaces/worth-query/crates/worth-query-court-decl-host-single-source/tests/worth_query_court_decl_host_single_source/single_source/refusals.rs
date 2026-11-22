//! A body action keeps every installation fence: contribution ownership,
//! a schema-owned body, and one owner per binding.
use super::program::{install, MISROUTE_PIN};
use super::schema::*;
use super::tasks::{RenameBinding, Renamed, TaskFeature, TasksDefinition, OMIT_BODY};
use std::cell::Cell;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_installation::WorthQueryApplicationPreparationDenial, primary_graph::*,
    worth_query_application_contribution,
};

fn refused_installation(switch: &'static std::thread::LocalKey<Cell<bool>>) -> WorthQueryPrimaryGraphInstallationDenial {
    switch.with(|value| value.set(true));
    let result = install();
    switch.with(|value| value.set(false));
    match result {
        Err(WorthQueryApplicationPreparationDenial::Contributions(denial)) => denial,
        Err(other) => panic!("refused outside contribution setup: {other:?}"),
        Ok(_) => panic!("the misconfigured board installed"),
    }
}

#[test]
fn a_body_action_configured_by_a_contribution_that_does_not_own_it_is_refused() {
    let denial = refused_installation(&MISROUTE_PIN);
    assert_eq!(
        denial.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::ContributionMemberMismatch,
        "{denial:?}"
    );
    assert_eq!(denial.subject(), "single-source.flags.pin.v1.binding");
}

#[test]
fn a_body_action_whose_operation_has_no_authored_body_is_refused() {
    let denial = refused_installation(&OMIT_BODY);
    assert_eq!(
        denial.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::OperationBodyUnavailable,
        "{denial:?}"
    );
}

worth_query_application_contribution! {
    pub feature DuplicateRename<Schema> for TaskFeature => DuplicateRuntime {
        schema: |schema| { schema };
        body_action(RenameBinding, Renamed) {
            operation: super::tasks::rename_definition();
        };
    }
}

fn board() -> ApplicationSchemaDeclarationBuilder<Schema> {
    ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(Board)
        .expect("the board fragment lowers")
        .principal_binding(PrincipalBinding::reference())
}

#[test]
fn a_body_action_declared_by_two_features_collides() {
    let declared = TasksDefinition::declare_members(board());
    assert!(declared.clone().build().is_ok());
    assert_eq!(
        DuplicateRename::declare_members(declared).build().unwrap_err(),
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
}
