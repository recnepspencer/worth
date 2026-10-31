//! A public authored root inverse survives same-kind collateral retirement.
use super::{
    authentication::*, authored_identity_courts as lifecycle, authored_identity_model::*, host,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

pub fn run() {
    let application = host::install();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let created = request
        .advanced()
        .mutate(Create {
            key: "numeric".to_owned(),
            name: "root".to_owned(),
            payload: 41,
            inverse_target: None,
            collateral: Some("child".to_owned()),
        })
        .without_source()
        .idempotency(&"create-root-child".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: created, ..
    } = created
    else {
        panic!("create root and derived child: {created:?}")
    };
    let original = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created>()
        .unwrap()
        .entity_id();
    let original_child = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<CollateralCreated>()
        .unwrap()
        .expect("created collateral output")
        .entity_id();
    let root_before = lifecycle::read(&application, "root");
    let child_before = lifecycle::read(&application, "child");
    assert_eq!(root_before[0].primary, 41);
    assert_eq!(child_before, root_before);
    let retired = request
        .advanced()
        .mutate(Retire {
            key: "numeric".to_owned(),
            target: "root".to_owned(),
            wrong_restore_key: false,
            collateral: Some("child".to_owned()),
        })
        .without_source()
        .idempotency(&"retire-root-child".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: retired, ..
    } = retired
    else {
        panic!("unique common root must retire with collateral: {retired:?}")
    };
    assert_eq!(
        retired
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<Retired>()
            .unwrap()
            .entity_id(),
        original
    );
    assert_eq!(
        retired
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<CollateralRetired>()
            .unwrap()
            .expect("retired collateral output")
            .entity_id(),
        original_child
    );
    lifecycle::absent(&application, "root");
    lifecycle::absent(&application, "child");
    let restored = super::tree_reapplication::reverse(&application, &retired, "restore-root-child");
    let successor = restored
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created>()
        .unwrap()
        .entity_id();
    assert_ne!(successor, original);
    let successor_child = restored
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<CollateralCreated>()
        .unwrap()
        .expect("restored collateral output")
        .entity_id();
    assert_ne!(successor_child, original_child);
    assert_eq!(lifecycle::read(&application, "root"), root_before);
    assert_eq!(lifecycle::read(&application, "child"), child_before);
    let alternative = super::tree_reapplication::alternative(&application, &retired, &restored, 1);
    let prepared = request
        .corrections()
        .reapply(&alternative)
        .idempotency("reapply-root-child")
        .limits(super::tree_reapplication::limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("authored retirement reapplication must prepare")
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("reapplied root retirement: {outcome:?}")
    };
    assert_eq!(
        receipt
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<Retired>()
            .unwrap()
            .entity_id(),
        successor
    );
    assert_eq!(
        receipt
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<CollateralRetired>()
            .unwrap()
            .expect("reapplied collateral output")
            .entity_id(),
        successor_child
    );
    lifecycle::absent(&application, "root");
    lifecycle::absent(&application, "child");
    assert_eq!(host::read(&application)[0].primary, 5);
}
