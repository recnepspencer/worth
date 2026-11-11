use super::branch_history_publications::{create_entity_outcome_on_branch, fork};
use super::merge_proof_journey::*;
use std::collections::BTreeMap;
use worth_foundational::{CanonicalBasisLocus, CanonicalBasisValue, InternedString};
use worth_relational::facade::{history::*, merge::*, runtime::RelationalRuntime};
use worth_testkit_relational::durable_medium::TestDurableMedium;
#[test]
fn merge_proof_packet_canonical_basis_parity_survives_publication_and_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);
    fork(execution, &runtime, "alt");
    create_entity_outcome_on_branch(execution, &runtime, "alt-only", BranchId("alt".into()));
    let outcome = execute_merge(execution, &runtime);
    let live_packet = outcome.execution_summary.proof_packet().clone();
    let live_basis = lower_packet(&runtime, &live_packet);
    let published_packet =
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id)
            .execution_summary
            .proof_packet
            .clone();
    let published_basis = lower_packet(&runtime, &published_packet);
    let (_recovery, recovered) = checkpoint_and_recover(execution, &runtime, &medium);
    let recovered_packet =
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id)
            .execution_summary
            .proof_packet
            .clone();
    let recovered_basis = lower_packet(&recovered, &recovered_packet);

    assert_eq!(live_basis, published_basis);
    assert_eq!(live_basis, recovered_basis);
    assert_eq!(entry_map(&live_basis), entry_map(&published_basis));
    assert_eq!(entry_map(&live_basis), entry_map(&recovered_basis));
}
fn lower_packet(
    runtime: &RelationalRuntime,
    packet: &worth_relational::facade::merge::RelationalMergeProofPacket,
) -> RelationalMergeProofPacketCanonicalBasis {
    let outcome = runtime
        .merge()
        .lower_merge_proof_packet_to_foundational_canonical_basis(packet);
    let worth_proof::TransitionOutcome::Success(basis) = outcome else {
        panic!("phase 6 canonical lowering must succeed for retained merge proof packet");
    };
    basis
}
fn entry_map(basis: &RelationalMergeProofPacketCanonicalBasis) -> BTreeMap<String, String> {
    basis
        .entries()
        .iter()
        .map(|entry| {
            (
                entry_locus(entry.locus()),
                canonical_value_text(entry.value()),
            )
        })
        .collect()
}
fn entry_locus(locus: &CanonicalBasisLocus) -> String {
    match locus {
        CanonicalBasisLocus::Named(InternedString::Raw(value)) => value.clone(),
        CanonicalBasisLocus::Named(InternedString::Symbol(symbol)) => {
            format!("symbol:{}", symbol.0)
        }
        other => format!("{other:?}"),
    }
}
fn canonical_value_text(value: &CanonicalBasisValue) -> String {
    match value {
        CanonicalBasisValue::ExactText(InternedString::Raw(text)) => format!("text:{text}"),
        CanonicalBasisValue::ExactText(InternedString::Symbol(symbol)) => {
            format!("text:symbol:{}", symbol.0)
        }
        CanonicalBasisValue::BytesDigest(digest) => {
            let hex = digest
                .bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            format!("digest:{hex}")
        }
        CanonicalBasisValue::UnsignedInteger { value, .. } => format!("u128:{value}"),
        other => format!("unexpected:{other:?}"),
    }
}
