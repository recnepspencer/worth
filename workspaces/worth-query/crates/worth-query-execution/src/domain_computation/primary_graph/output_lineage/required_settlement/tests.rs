//! An exact accepted-row lookup ends in a stop or in a reason, never one as
//! the other: a stop denies the request, and a reason verifies in full.

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

use crate::domain_computation::primary_graph::application_output_demand::{
    ReadyCompletion, WorthQueryAcceptedOutputAuthority, WorthQueryCompletedOutputDemand,
    WorthQueryOutputReadinessDeliveryEvidence,
};
use crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application;

use super::super::{invalidation::InvalidationEditAdmission, WorthQueryApplicationOutputLineage};
use super::RequiredSettlementStop;

fn committed() -> (u64, ApplicationSchemaBindingIdentity, ReadyCompletion) {
    let receipt = committed_recoverable_application();
    let runtime_authority = receipt.principal_scope().runtime_authority();
    let schema = receipt.principal_scope().binding_identity().clone();
    let completion = ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
        readiness: WorthQueryOutputReadinessDeliveryEvidence::for_test(),
        resources: None,
    });
    (runtime_authority, schema, completion)
}

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 1024 * 1024,
    })
}

#[test]
fn a_foreign_accepted_authority_is_a_stop_not_a_reason() {
    let (runtime_authority, schema, completion) = committed();
    let resolved = WorthQueryApplicationOutputLineage::default().resolve_required_settlement(
        runtime_authority.wrapping_add(1),
        &schema,
        &completion,
        &mut admission(64),
    );
    assert_eq!(resolved.err(), Some(RequiredSettlementStop::Foreign));
}

#[test]
fn an_exhausted_lookup_is_a_stop_not_a_reason() {
    let (runtime_authority, schema, completion) = committed();
    let resolved = WorthQueryApplicationOutputLineage::default().resolve_required_settlement(
        runtime_authority,
        &schema,
        &completion,
        &mut admission(1),
    );
    assert!(matches!(
        resolved,
        Err(RequiredSettlementStop::Admission(
            CompanionPreflightStop::WorkExhausted { .. }
        ))
    ));
}

#[test]
fn an_uncertified_row_is_a_reason_not_a_stop() {
    let (runtime_authority, schema, completion) = committed();
    let resolved = WorthQueryApplicationOutputLineage::default().resolve_required_settlement(
        runtime_authority,
        &schema,
        &completion,
        &mut admission(1024),
    );
    assert!(matches!(resolved, Ok(Ok(None) | Err(_))));
}
