use serde::{Deserialize, Serialize};
use worth_foundational::facade::AuthoritativeRecordAspectState;

use super::CanonicalCommitEnvelope;
use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor};
use crate::identity::data::EntityId;
use crate::transactions::data::{RecordRef, TransactionCommitError};

mod payload_coverage;
mod relations;
use payload_coverage::{validate_created_output_payload, OutputPayloadCoverageFailure};
use relations::CanonicalRequiredOutputRelation;

/// Descriptive predecessor material, readmitted only against performed canonical
/// ancestry, exact native operations and the current physical payload.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanonicalRequiredOutputOrigin {
    pub(crate) commit: crate::history::data::RelationalCommitReceipt,
    pub(crate) basis: RelationalBranchBasisDescriptor,
    pub(crate) producer: String,
    pub(crate) dependency_identity: [u8; 32],
    pub(crate) outputs: Vec<EntityId>,
    // Strict current format: absence is not converted into relation authority.
    pub(crate) relations: Vec<CanonicalRequiredOutputRelation>,
    #[serde(skip)]
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl PartialEq for CanonicalRequiredOutputOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.commit == other.commit
            && self.basis == other.basis
            && self.producer == other.producer
            && self.dependency_identity == other.dependency_identity
            && self.outputs == other.outputs
            && self.relations == other.relations
    }
}
impl Eq for CanonicalRequiredOutputOrigin {}

impl CanonicalCommitEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn install_required_output_origin(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        basis: &AdmittedRelationalBranchBasis,
        created: &crate::transactions::data::CommitCreatedEntityBindings,
        created_relations: &crate::transactions::data::CommitCreatedRelationBindings,
        state: &crate::storage::overlay::WorkingState,
        admission: crate::mvcc::RequiredOutputPublicationAdmission,
    ) -> Result<(), TransactionCommitError> {
        let result = execution
            .in_scope(|lease| {
                worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                    .run_with_checkpoint(lease, None, |context| {
                        self.install_metered(
                            runtime,
                            basis,
                            created,
                            created_relations,
                            state,
                            admission,
                            lease,
                            context,
                        )
                    })
            })
            .map_err(origin_request)?
            .map_err(origin_request)?
            .0;
        execution.consult().map_err(origin_request)?;
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn install_metered(
        &mut self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        basis: &AdmittedRelationalBranchBasis,
        created: &crate::transactions::data::CommitCreatedEntityBindings,
        created_relations: &crate::transactions::data::CommitCreatedRelationBindings,
        state: &crate::storage::overlay::WorkingState,
        admission: crate::mvcc::RequiredOutputPublicationAdmission,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        let crate::mvcc::RequiredOutputPublicationAdmission {
            installed,
            proof,
            targets,
        } = admission;
        context.checkpoint(1).map_err(origin_request)?;
        let digest = basis.inner.root.schema_authority().authority_digest();
        if installed.runtime_instance_id != runtime.runtime_instance_id()
            || installed.custom_invariant_generation
                != runtime.schema_contract_runtime.custom_invariant_generation
            || installed.schema_digest != digest
            || self.schema_transition.is_some()
            || self.schema_continuation_descriptor.is_some()
            || self.schema_reconciliation_descriptor.is_some()
            || !installed.authority.admits_required_output(&proof)
            || self.branch_context != *basis.descriptor().branch_id()
        {
            return Err(crate::mvcc::origin_denial(
                "required-output publication installation changed before native preparation",
            ));
        }
        let bytes = origin_retention_bytes(
            self,
            basis,
            proof.producer().len(),
            targets.len(),
            created_relations.iter().len(),
        )?;
        context.checkpoint(bytes).map_err(origin_request)?;
        let memory = worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
            .map_err(|denial| origin_request(crate::execution::denial::lease_denial(denial)))?;
        // Stable-reference fallback owns one temporary raw key during lookup.
        // Its simultaneous scratch width is the largest actual selected key.
        let mut lookup_bytes = 0;
        for target in &targets {
            context.checkpoint(1).map_err(origin_request)?;
            lookup_bytes = lookup_bytes.max(target.client_key.as_raw_str().map_or(0, str::len));
        }
        let _lookup_memory = worth_execution::ExecutionMemoryReservation::reserve_in_scope(
            lease,
            u64::try_from(lookup_bytes).map_err(|_| {
                origin_request(
                    crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                )
            })?,
        )
        .map_err(|denial| origin_request(crate::execution::denial::lease_denial(denial)))?;
        let mut outputs = Vec::with_capacity(targets.len());
        for target in targets {
            context
                .checkpoint(created.resolve_work_bound(&target).ok_or_else(|| {
                    origin_request(
                        crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                    )
                })?)
                .map_err(origin_request)?;
            let id = created.resolve(&target).ok_or_else(|| {
                crate::mvcc::origin_denial(
                    "required-output target was not created by this native mutation",
                )
            })?;
            let partition = state.partitions.get(&id.partition_id).ok_or_else(|| {
                crate::mvcc::origin_denial("required-output target partition is absent")
            })?;
            let record = partition
                .entity_arena
                .get(&id)
                .filter(|record| record.is_live())
                .ok_or_else(|| crate::mvcc::origin_denial("required-output target is not live"))?;
            let Some(value) = record.extra().authoritative_aspect_state.as_ref() else {
                continue;
            };
            for (_, present) in value.aspects().entries() {
                context
                    .checkpoint(present.semantic_byte_width() as u64)
                    .map_err(origin_request)?;
            }
            match validate_created_output_payload(self, id, value, context) {
                Ok(()) => {}
                Err(OutputPayloadCoverageFailure::Request(cause)) => {
                    return Err(origin_request(cause))
                }
                Err(OutputPayloadCoverageFailure::Unsupported) => continue,
                Err(OutputPayloadCoverageFailure::Corrupt(detail)) => {
                    return Err(crate::mvcc::origin_denial(detail))
                }
            }
            outputs.push(id);
        }
        let sort_work = outputs
            .len()
            .checked_mul((usize::BITS - outputs.len().leading_zeros()) as usize + 1)
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(|| {
                origin_request(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                )
            })?;
        context.checkpoint(sort_work).map_err(origin_request)?;
        outputs.sort_unstable();
        outputs.dedup();
        let relations = relations::capture_created_relations(
            self,
            created,
            created_relations,
            state,
            &outputs,
            lease,
            context,
        )?;
        self.required_output_origin = (!outputs.is_empty()).then(|| {
            std::sync::Arc::new(CanonicalRequiredOutputOrigin {
                commit: self.commit.clone(),
                basis: basis.descriptor().clone(),
                producer: proof.producer().to_owned(),
                dependency_identity: *proof.dependency_identity(),
                outputs,
                relations,
                _funding: Some(memory),
            })
        });
        Ok(())
    }

    pub(crate) fn readmit_required_output_payload(
        &self,
        id: EntityId,
        payload: &AuthoritativeRecordAspectState,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), crate::history::data::CodecRetirementClosureStop> {
        let origin = self
            .required_output_origin
            .as_ref()
            .ok_or("present output has no installed producer publication")?;
        let words = self
            .commit
            .parents
            .len()
            .saturating_add(origin.commit.parents.len())
            .saturating_add(self.patch.authoritative_record_patches.len())
            .saturating_add(usize::BITS as usize - origin.outputs.len().leading_zeros() as usize)
            .saturating_add(1);
        let text = self
            .commit
            .branch_id
            .0
            .len()
            .saturating_add(origin.commit.branch_id.0.len())
            .saturating_add(self.branch_context.0.len())
            .saturating_add(origin.basis.branch_id().0.len())
            .saturating_add(origin.basis.reference().branch_id().as_str().len())
            .saturating_add(
                self.branch_cell_checkpoint
                    .as_ref()
                    .map_or(0, |cell| cell.observation.branch_id().as_str().len()),
            );
        context
            .checkpoint(
                words
                    .saturating_mul(std::mem::size_of::<u64>())
                    .saturating_add(text) as u64,
            )
            .map_err(|cause| {
                crate::history::data::CodecRetirementClosureStop::Request(cause.into())
            })?;
        if origin.commit != self.commit
            || origin.producer.is_empty()
            || origin.outputs.binary_search(&id).is_err()
            || origin.basis.branch_id() != &self.branch_context
            || self
                .branch_cell_checkpoint
                .as_ref()
                .map(|cell| &cell.observation)
                != Some(origin.basis.reference())
            || !self
                .patch
                .authoritative_record_patches
                .iter()
                .any(|record| record.target == RecordRef::Entity(id))
        {
            return Err("copied or mismatched required-output publication origin".into());
        }
        validate_created_output_payload(self, id, payload, context).map_err(|failure| match failure
        {
            OutputPayloadCoverageFailure::Request(cause) => {
                crate::history::data::CodecRetirementClosureStop::Request(cause)
            }
            OutputPayloadCoverageFailure::Unsupported => {
                crate::history::data::CodecRetirementClosureStop::Payload(
                    "required output contains unsupported or unclassified present cells".into(),
                )
            }
            OutputPayloadCoverageFailure::Corrupt(detail) => {
                crate::history::data::CodecRetirementClosureStop::Payload(detail)
            }
        })
    }
}

fn origin_request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}

fn origin_retention_bytes(
    envelope: &CanonicalCommitEnvelope,
    basis: &AdmittedRelationalBranchBasis,
    producer_bytes: usize,
    targets: usize,
    relations: usize,
) -> Result<u64, TransactionCommitError> {
    let parent_bytes = basis
        .reference()
        .target()
        .as_basis()
        .map_or(0, |target| target.parent_commit_ids().len())
        .checked_mul(std::mem::size_of::<u64>());
    let bytes = std::mem::size_of::<CanonicalRequiredOutputOrigin>()
        .checked_add(2 * std::mem::size_of::<usize>())
        .and_then(|bytes| bytes.checked_add(producer_bytes))
        .and_then(|bytes| bytes.checked_add(basis.descriptor().branch_id().0.len()))
        .and_then(|bytes| bytes.checked_add(basis.reference().branch_id().as_str().len()))
        .and_then(|bytes| bytes.checked_add(envelope.commit.branch_id.0.len()))
        .and_then(|bytes| {
            bytes.checked_add(
                envelope
                    .commit
                    .parents
                    .len()
                    .checked_mul(std::mem::size_of::<crate::history::data::CommitId>())?,
            )
        })
        .and_then(|bytes| bytes.checked_add(parent_bytes?))
        .and_then(|bytes| bytes.checked_add(targets.checked_mul(std::mem::size_of::<EntityId>())?))
        .and_then(|bytes| {
            bytes.checked_add(
                relations.checked_mul(std::mem::size_of::<CanonicalRequiredOutputRelation>())?,
            )
        })
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| {
            origin_request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
    Ok(bytes)
}
