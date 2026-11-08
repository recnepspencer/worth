//! A real writer receives only its current predecessor or genuine route absence.
use super::*;
use crate::{
    history::data::CanonicalRequiredOutputOrigin,
    publication::data::AuthoredRetirementRefusal as Cause,
};

pub(super) fn admit(
    candidate: &RelationalAuthoredCoupledCandidate<'_>,
    execution: &ExecutionRequest<'_, '_>,
    publication: &RequiredOutputPublication,
) -> Result<Option<Arc<CanonicalRequiredOutputOrigin>>, TransactionCommitError> {
    let binding = candidate.binding;
    let meaning = publication
        .producing_meaning()
        .ok_or_else(|| part::reject(Cause::ForeignCoupledOwner, binding.owners.source.subject()))?;
    let subject = meaning.source_scope;
    let foreign = || part::reject(Cause::ForeignCoupledOwner, subject);
    if !publication.admitted_by(&binding.installed) {
        return Err(foreign());
    }
    let ordinary = matches!(binding.owners.source, super::super::CoupledSource::Edit(_));
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
                        candidate.runtime.services.runtime_instance_id(),
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
                            for owner in &binding.owners.owners {
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
                .ok_or_else(|| part::reject(Cause::ExtraCoupledOwner, subject))?;
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
                    candidate
                        .mutation
                        .outcome
                        .created_entities
                        .created_here(subject, context)
                })
                .map_err(custody::execution_stop)?
                .0
                .map_err(custody::execution_stop)?;
            // The allocation owner proves this full-generation source is new
            // in D. It has no C record to ask the current roster reader about.
            if !created
                && custody::readmit_successor(
                    candidate.runtime.services.runtime_instance_id(),
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
    execution
        .run_native_with_checkpoint(|context| {
            part::require_unseen_route(context, &candidate.parts, meaning)
        })
        .map_err(custody::execution_stop)?
        .0?;
    Ok(prior)
}
