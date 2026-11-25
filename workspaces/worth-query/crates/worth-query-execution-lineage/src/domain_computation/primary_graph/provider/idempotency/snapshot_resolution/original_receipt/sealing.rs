//! Only the two concrete owner input classes can mint original receipt data.
use super::*;
use worth_proof::QueryReadmitted;
use worth_runtime_world::facade::{WorldObservedHomePublication, WorldObservedMemoryPublication};

#[derive(Debug, Eq, PartialEq)]
pub enum PublicationOrigin {
    Home {
        position: u64,
        digest: [u8; 32],
        publisher: HomeIncarnation,
    },
    MemoryOnly,
}

#[derive(Debug)]
pub struct OriginalReceiptRows {
    pub binding: RecordedOriginalBinding,
    pub before_component: RecordedRelationalComponentReference,
    pub after_component: RecordedRelationalComponentReference,
    pub outbox: Option<WorthQueryRecoveredDispatchOutbox>,
    pub causality: Option<WorthQueryRecoveredAftermathCausality>,
    pub funding: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug)]
pub struct OriginalReceiptInputs {
    pub publication: WorldObservedHomePublication,
    pub rows: OriginalReceiptRows,
}
impl worth_proof::AuthorityMarker for OriginalReceiptInputs {}

#[derive(Debug)]
pub struct MemoryOriginalReceiptInputs {
    pub publication: WorldObservedMemoryPublication,
    pub rows: OriginalReceiptRows,
}
impl worth_proof::AuthorityMarker for MemoryOriginalReceiptInputs {}

impl WorthQueryRecoveredApplicationCommitReceipt {
    pub(crate) fn from_readmitted(
        original: QueryReadmitted<OriginalReceiptInputs>,
    ) -> Result<Self, super::super::WorthQueryProviderIdempotencyResolutionDenial> {
        original.consume(|original| {
            let old_head = RecordedProductBranchReference::Home(original.publication.before().clone());
            let new_head = RecordedProductBranchReference::Home(original.publication.after().clone());
            let recorded = from_rows(
                original.rows,
                RecordedPublication {
                    old_head,
                    new_head,
                    attempt: RecordedCompositePublicationAttemptIdentity::Home(
                        original.publication.attempt_identity(),
                    ),
                    origin: PublicationOrigin::Home {
                        position: original.publication.journal_position(),
                        digest: *original.publication.artifact_digest(),
                        publisher: original.publication.admitted_incarnation(),
                    },
                    semantic: original.publication.recovered_semantic_result().clone(),
                    native_position: original.publication.native_patch_position(),
                    native_digest: *original.publication.native_artifact_digest(),
                },
            );
            drop(original.publication);
            Ok(Self {
                original: Arc::new(recorded),
            })
        })
    }

    pub(crate) fn from_memory(
        original: QueryReadmitted<MemoryOriginalReceiptInputs>,
    ) -> Result<Self, super::super::WorthQueryProviderIdempotencyResolutionDenial> {
        original.consume(|original| {
            let old_head =
                RecordedProductBranchReference::Memory(original.publication.before().clone());
            let new_head =
                RecordedProductBranchReference::Memory(original.publication.after().clone());
            let owner = new_head
                .memory()
                .ok_or(super::super::WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?
                .owner();
            let native = original.publication.native();
            let recorded = from_rows(
                original.rows,
                RecordedPublication {
                    old_head,
                    new_head,
                    attempt: RecordedCompositePublicationAttemptIdentity::Memory {
                        owner,
                        ordinal: original.publication.attempt_identity().ordinal(),
                    },
                    origin: PublicationOrigin::MemoryOnly,
                    semantic: native.semantic_result().clone(),
                    native_position: native.position(),
                    native_digest: *native.artifact_digest(),
                },
            );
            drop(original.publication);
            Ok(Self {
                original: Arc::new(recorded),
            })
        })
    }
}

struct RecordedPublication {
    old_head: RecordedProductBranchReference,
    new_head: RecordedProductBranchReference,
    attempt: RecordedCompositePublicationAttemptIdentity,
    origin: PublicationOrigin,
    semantic: RecoveredRelationalSemanticResult,
    native_position: PatchStreamPosition,
    native_digest: [u8; 32],
}

fn from_rows(rows: OriginalReceiptRows, publication: RecordedPublication) -> RecordedReceipt {
    RecordedReceipt {
        binding: rows.binding,
        before_component: rows.before_component,
        after_component: rows.after_component,
        outbox: rows.outbox,
        causality: rows.causality,
        _funding: rows.funding,
        old_head: publication.old_head,
        new_head: publication.new_head,
        attempt: publication.attempt,
        origin: publication.origin,
        semantic: publication.semantic,
        native_position: publication.native_position,
        native_digest: publication.native_digest,
    }
}
