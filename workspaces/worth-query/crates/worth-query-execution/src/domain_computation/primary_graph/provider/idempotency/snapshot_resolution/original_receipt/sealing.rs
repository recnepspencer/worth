//! Only the two concrete owner input classes can mint original receipt data.
use super::*;
use worth_proof::QueryReadmitted;
use worth_runtime_world::facade::{
    ProductBranchReferenceSnapshot, WorldObservedMemoryPublication, WorldReadmittedPublication,
};

#[derive(Debug, Eq, PartialEq)]
pub(super) enum PublicationOrigin {
    Home {
        position: u64,
        digest: [u8; 32],
        publisher: HomeIncarnation,
    },
    MemoryOnly,
}

#[derive(Debug)]
pub(in super::super) struct OriginalReceiptRows {
    pub(in super::super) binding: RecordedOriginalBinding,
    pub(in super::super) before_component: RecordedRelationalComponentReference,
    pub(in super::super) after_component: RecordedRelationalComponentReference,
    pub(in super::super) outbox: Option<WorthQueryRecoveredDispatchOutbox>,
    pub(in super::super) causality: Option<WorthQueryRecoveredAftermathCausality>,
    pub(in super::super) funding: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug)]
pub(in super::super) struct OriginalReceiptInputs {
    pub(in super::super) publication: WorldReadmittedPublication,
    pub(in super::super) rows: OriginalReceiptRows,
}
impl worth_proof::AuthorityMarker for OriginalReceiptInputs {}

#[derive(Debug)]
pub(in super::super) struct MemoryOriginalReceiptInputs {
    pub(in super::super) publication: WorldObservedMemoryPublication,
    pub(in super::super) rows: OriginalReceiptRows,
}
impl worth_proof::AuthorityMarker for MemoryOriginalReceiptInputs {}

impl WorthQueryRecoveredApplicationCommitReceipt {
    pub(in super::super) fn from_readmitted(
        original: QueryReadmitted<OriginalReceiptInputs>,
    ) -> Result<Self, super::super::WorthQueryProviderIdempotencyResolutionDenial> {
        original.consume(|original| {
            let old_head = recorded_head(original.publication.before())?;
            let new_head = recorded_head(original.publication.after())?;
            let owner = new_head.durable().ok_or(super::super::WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?.branch().owner();
            let recorded = from_rows(original.rows, RecordedPublication {
                old_head, new_head,
                attempt: RecordedCompositePublicationAttemptIdentity::Home(worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(owner, original.publication.attempt_identity().ordinal())),
                origin: PublicationOrigin::Home { position: original.publication.journal_position(), digest: *original.publication.artifact_digest(), publisher: original.publication.admitted_incarnation() },
                semantic: original.publication.recovered_semantic_result().clone(),
                native_position: original.publication.native_patch_position(),
                native_digest: *original.publication.native_artifact_digest(),
            });
            drop(original.publication);
            Ok(Self { original: Arc::new(recorded) })
        })
    }

    pub(in super::super) fn from_memory(
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

fn recorded_head(
    value: &ProductBranchReferenceSnapshot,
) -> Result<
    RecordedProductBranchReference,
    super::super::WorthQueryProviderIdempotencyResolutionDenial,
> {
    use super::super::WorthQueryProviderIdempotencyResolutionDenial::Unavailable;
    use worth_foundational::facade::{
        DurableCompositeCommitIdentity, DurableProductBranchIdentity,
        DurableProductBranchIncarnation, DurableProductBranchReference,
        DurableProductReferenceGeneration, MemoryProductBranchReference,
    };
    let owner = value.branch_identity().owner_identity();
    match owner.durable() {
        Some(owner) => DurableProductBranchReference::from_recorded(
            DurableProductBranchIdentity::from_recorded(
                owner,
                value.branch_identity().name().as_str().to_owned(),
            )
            .map_err(|_| Unavailable)?,
            DurableProductBranchIncarnation::from_recorded(
                owner,
                value.lifecycle_incarnation().ordinal(),
            ),
            DurableProductReferenceGeneration::from_recorded(value.reference_generation().get()),
            DurableCompositeCommitIdentity::from_recorded(owner, value.selected_commit().ordinal()),
        )
        .map(RecordedProductBranchReference::Home)
        .map_err(|_| Unavailable),
        None => MemoryProductBranchReference::from_observed(
            owner.get(),
            value.branch_identity().name().as_str().to_owned(),
            value.lifecycle_incarnation().ordinal(),
            value.reference_generation().get(),
            value.selected_commit().ordinal(),
        )
        .map(RecordedProductBranchReference::Memory)
        .map_err(|_| Unavailable),
    }
}
