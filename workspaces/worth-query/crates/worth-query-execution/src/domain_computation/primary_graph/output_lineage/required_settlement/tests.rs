//! An exact accepted-row lookup ends in a stop or in a reason, never one as
//! the other: a stop denies the request, and a reason verifies in full.

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use crate::domain_computation::primary_graph::application_output_demand::ReadyCompletion;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryCompletedOutputDemand;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence;
use crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application;

use super::super::WorthQueryApplicationOutputLineage;
use super::RequiredSettlementStop;

fn committed() -> (u64, ApplicationSchemaBindingIdentity, ReadyCompletion) {
    let receipt = committed_recoverable_application();
    let runtime_authority = receipt.principal_scope().runtime_authority();
    let schema = receipt.principal_scope().binding_identity().clone();
    let completion = ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt),
        readiness: WorthQueryOutputReadinessDeliveryEvidence::for_test(),
    });
    (runtime_authority, schema, completion)
}

#[test]
fn a_foreign_accepted_authority_is_a_stop_not_a_reason() {
    let (runtime_authority, schema, completion) = committed();
    let resolved = WorthQueryApplicationOutputLineage::default().resolve_required_settlement(
        runtime_authority.wrapping_add(1),
        &schema,
        &completion,
    );
    assert_eq!(resolved.err(), Some(RequiredSettlementStop::Foreign));
}

#[test]
fn an_uncertified_row_is_a_reason_not_a_stop() {
    let (runtime_authority, schema, completion) = committed();
    let resolved = WorthQueryApplicationOutputLineage::default().resolve_required_settlement(
        runtime_authority,
        &schema,
        &completion,
    );
    assert!(matches!(resolved, Ok(Ok(None) | Err(_))));
}
