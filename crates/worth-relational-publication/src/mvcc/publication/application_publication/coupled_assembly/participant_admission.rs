//! A real writer receives only its current predecessor or genuine route absence.
use super::super::custody;
use super::super::RelationalRequiredOutputRouteAbsence;
use super::super::RequiredOutputPublication;
use super::super::RequiredOutputPublicationAdmission;
use super::producer_posture::RelationalCoupledProducerPosture;
use super::retained_assembly::{BoundAuthoredCoupling, CoupledPublicationPart};
use contracts::execution::request_stage::checkpoint;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
pub mod ordinary;
use super::refusals::{reject, reject_extra_owner};
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;

/// A first entry: its posture, and a route no earlier part has entered.
pub fn admit(
    binding: &BoundAuthoredCoupling,
    runtime_instance_id: u64,
    created_entities: &CommitCreatedEntityBindings,
    parts: &[CoupledPublicationPart],
    execution: &ExecutionRequest<'_, '_>,
    publication: &RelationalCoupledProducerPosture,
) -> Result<Option<Arc<CanonicalRequiredOutputOrigin>>, TransactionCommitError> {
    let prior = admit_posture(
        binding,
        runtime_instance_id,
        created_entities,
        execution,
        publication,
    )?;
    let meaning = publication.producing_meaning().expect("admitted writer");
    execution
        .run_native_with_checkpoint(|context| require_unseen_route(context, parts, meaning))
        .map_err(custody::execution_stop)?
        .0?;
    Ok(prior)
}

/// The posture alone, against C: a successor round of an entered route passes
/// exactly this and its chain proof instead of the unseen-route check.
pub fn admit_posture(
    binding: &BoundAuthoredCoupling,
    runtime_instance_id: u64,
    created_entities: &CommitCreatedEntityBindings,
    execution: &ExecutionRequest<'_, '_>,
    publication: &RelationalCoupledProducerPosture,
) -> Result<Option<Arc<CanonicalRequiredOutputOrigin>>, TransactionCommitError> {
    let meaning = publication
        .producing_meaning()
        .ok_or_else(|| reject(Cause::ForeignCoupledOwner, binding.owners.source.subject()))?;
    let subject = meaning.source_scope;
    let foreign = || reject(Cause::ForeignCoupledOwner, subject);
    if !publication.admitted_by(&binding.installed) {
        return Err(foreign());
    }
    let ordinary = binding.owners.source.is_edit();
    if let RelationalCoupledProducerPosture::Ordinary {
        admission,
        absence,
        meaning,
        roles,
    } = publication
    {
        ordinary::require(binding, execution, admission, absence, meaning, roles)?;
        return Ok(None);
    }
    let publication = publication.owned().expect("remaining owned participant");
    let prior = match publication {
        RequiredOutputPublication::Preserve { predecessor, .. } => {
            if !meaning.same_route(&predecessor.origin.meaning)
                || (!ordinary && !predecessor.origin.source_scope_owned)
            {
                return Err(foreign());
            }
            // Independent ordinary predecessors carry their real canonical seal.
            // The canonical installer's predecessor owner checks current position,
            // runtime, complete origin and relations before any publication.
            if ordinary && !predecessor.origin.source_scope_owned {
                let (producer, _) = publication.lineage_attribution().expect("admitted writer");
                if binding
                    .installed
                    .source_scope_ownership
                    .scope_kind(producer)
                    .is_some()
                {
                    return Err(foreign());
                }
                Some(Arc::clone(&predecessor.origin))
            } else {
                let expected = if ordinary {
                    let (producer, _) = publication.lineage_attribution().expect("admitted writer");
                    custody::readmit_successor(
                        runtime_instance_id,
                        execution,
                        &binding.installed,
                        &binding.owners.before,
                        producer,
                        meaning,
                    )?
                } else {
                    execution
                        .run_native_with_checkpoint(|context| {
                            let mut expected = None;
                            // A later edit stage's survivor republishes from its
                            // exact current predecessor, as a required owner does.
                            for owner in binding
                                .owners
                                .owners
                                .iter()
                                .chain(&binding.owners.survivors)
                            {
                                checkpoint(
                                    context,
                                    (meaning.family.len() + owner.origin.meaning.family.len() + 65)
                                        as u64,
                                )
                                .map_err(custody::execution_stop)?;
                                if owner.origin.meaning.same_route(meaning) {
                                    expected = Some(owner.clone());
                                }
                            }
                            Ok::<_, TransactionCommitError>(expected)
                        })
                        .map_err(custody::execution_stop)?
                        .0?
                }
                .ok_or_else(|| reject_extra_owner(
                    &binding.owners,
                    subject,
                    format!(
                        "site=preserve-owner-absent; family={}; requested_source={subject:?}; partition={:?}; producer={:?}",
                        meaning.family, meaning.source_partition.bytes(),
                        publication.lineage_attribution().map(|(producer, _)| producer),
                    ),
                ))?;
                if !expected.same_origin(predecessor) {
                    return Err(foreign());
                }
                Some(Arc::clone(&predecessor.origin))
            }
        }
        RequiredOutputPublication::Birth { absence, .. } => {
            if !ordinary {
                return Err(foreign());
            }
            let (producer, _) = publication.lineage_attribution().expect("admitted writer");
            let created = execution
                .run_native_with_checkpoint(|context| {
                    created_entities.created_here(subject, context)
                })
                .map_err(custody::execution_stop)?
                .0
                .map_err(custody::execution_stop)?;
            // The allocation owner proves this full-generation source is new
            // in D. It has no C record to ask the current roster reader about.
            if !created
                && custody::readmit_successor(
                    runtime_instance_id,
                    execution,
                    &binding.installed,
                    &binding.owners.before,
                    producer,
                    meaning,
                )?
                .is_some()
            {
                return Err(foreign());
            }
            execution
                .run_native_with_checkpoint(|context| {
                    RequiredOutputPublicationAdmission::birth_retirement_in_context(
                        &binding.installed,
                        &binding.owners.before,
                        meaning,
                        absence.as_ref(),
                        context,
                    )
                })
                .map_err(custody::execution_stop)?
                .0?;
            None
        }
        _ => unreachable!("only admitted producer variants have route meaning"),
    };
    Ok(prior)
}

pub fn require_unseen_route(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    parts: &[CoupledPublicationPart],
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<(), TransactionCommitError> {
    for part in parts {
        let prior = part
            .publication
            .producing_meaning()
            .expect("admitted producer route");
        checkpoint(
            context,
            (meaning.family.len() + prior.family.len() + 65) as u64,
        )
        .map_err(custody::execution_stop)?;
        if prior.same_route(meaning) {
            return Err(reject(Cause::RepeatedCoupledOwner, meaning.source_scope));
        }
    }
    Ok(())
}
