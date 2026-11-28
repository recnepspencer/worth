//! Typed exclusion evidence for the existing durable validation quote fixture.
use super::OwnerAllocation;
use contracts::diagnostics::data::artifacts::DiagnosticCode;
use contracts::identity::data::{EntityId, KindId, PartitionId};
use contracts::transactions::data::outcomes::commit_semantic_result::validation::RelationalInvariantCheckMeaning;
use contracts::validation::data::execution::{
    InvariantClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantReportedRule,
    InvariantVerdict,
};
use contracts::validation::data::groups::{InvariantCostClass, InvariantGroupSet};
use contracts::validation::data::results::{InvariantViolation, InvariantViolationFields};
use contracts::validation::data::rules::InvariantRule;
use serde::{Deserialize, Serialize};
use worth_relational_contracts::internal_port as contracts;

pub(super) fn assert_roundtrip_and_quote(quote: impl Fn(&[u8]) -> OwnerAllocation) {
    let short_id = "form";
    let long_id = "form-selection-λ";
    let short = quote(&roundtrip(short_id));
    let long = quote(&roundtrip(long_id));
    assert_eq!(
        long.retained,
        short.retained + (long_id.len() - short_id.len()) as u64
    );
    assert_eq!(long.transient, short.transient);
}

fn roundtrip(group_id: &str) -> Vec<u8> {
    let check = RelationalInvariantCheckMeaning {
        execution_point: InvariantExecutionPoint::CommitBoundary,
        failure_effect: InvariantFailureEffect::BlockCommit,
        rule: InvariantReportedRule::Native(InvariantRule::EntityAspectExclusion),
        witness: "exclusion".into(),
        groups: InvariantGroupSet::empty(),
        declared_cost: InvariantCostClass::Touched,
        custom_provenance: None,
        verdict: InvariantVerdict::Violation(InvariantViolation {
            class: InvariantClass::CommitBoundary,
            code: DiagnosticCode::InvariantViolation,
            detail: "multiple aspects present".into(),
            fields: InvariantViolationFields::EntityAspectExclusion {
                entity_id: EntityId::new(PartitionId(1), 2, 3),
                kind_id: KindId(4),
                group_id: group_id.into(),
                required: true,
                active_count: 2,
            },
        }),
    };
    #[derive(Serialize, Deserialize)]
    struct SelectedValidation {
        checks: [RelationalInvariantCheckMeaning; 1],
    }
    let original = SelectedValidation { checks: [check] };
    let bytes = rmp_serde::to_vec_named(&original).unwrap();
    let decoded = rmp_serde::from_slice::<SelectedValidation>(&bytes).unwrap();
    assert_eq!(decoded.checks, original.checks);
    bytes
}
