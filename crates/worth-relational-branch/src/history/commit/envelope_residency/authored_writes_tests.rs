//! Public #382: a span install only ever widens the target's generation.
use super::RelationalCommitEnvelopeResidency;
use crate::history::access::relevant_writes::{generation::AuthoredWriteGeneration, run};
use crate::history::data::CommitId;
use std::sync::Arc;

fn span(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    commits: std::ops::RangeInclusive<u64>,
) -> AuthoredWriteGeneration {
    run(execution, |context| {
        let mut generation = AuthoredWriteGeneration::default();
        for commit in commits {
            generation.advance(execution, context, CommitId(commit))?;
        }
        Ok(generation)
    })
    .unwrap()
}

#[test]
fn a_span_install_never_narrows_the_installed_generation() {
    use crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update_for_test as envelope;
    use crate::identity::data::{EntityId, PartitionId};
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let original =
        crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
            &execution,
            Arc::new(envelope(
                crate::transactions::data::primitives::RecordRef::Entity(entity),
                entity,
            )),
        )
        .unwrap();
    let residency = RelationalCommitEnvelopeResidency::resident(original);
    let covers = |commit| {
        residency
            .authored_write_generation()
            .is_some_and(|generation| generation.prefix.get(&CommitId(commit)).is_some())
    };
    assert!(
        residency.install_authored_write_span(CommitId(5), span(execution, 5..=7)),
        "an empty target installs"
    );
    assert!(
        residency.install_authored_write_span(CommitId(3), span(execution, 3..=7)),
        "an older original widens"
    );
    assert!(
        !residency.install_authored_write_span(CommitId(5), span(execution, 5..=7)),
        "a narrower seed that lost the race is discarded"
    );
    assert!(
        covers(3) && covers(7),
        "the wider generation stays installed"
    );
}
