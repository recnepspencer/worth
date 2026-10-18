//! Re-admit complete original output association on the recovered source state.
use super::{
    charge, intent_writes_entity, memory_overflow, request, require_linear_history, reserve,
    CodecRetirementClosureStop as Stop,
};
use crate::{
    branch::AdmittedRelationalBranchBasis,
    identity::data::{EntityId, KindId},
    runtime::RelationalRuntime,
    transactions::data::RecordRef,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, MapKernelContext};
type Origin =
    std::sync::Arc<super::super::canonical_commit_envelope::CanonicalRequiredOutputOrigin>;
pub(crate) fn validate_codec_output_correspondence(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    producer: &str,
    dependency: &[u8; 32],
    roles: &[(EntityId, KindId)],
) -> Result<(), Stop> {
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                if basis.descriptor().runtime_instance_id() != runtime.runtime_instance_id()
                    || !basis.is_current()
                {
                    return Err(
                        "output correspondence requires the exact current recovered basis".into(),
                    );
                }
                if roles.is_empty() {
                    return Err("output correspondence correspondence is empty".into());
                }
                let branch = runtime
                    .read_truth()
                    .project_observation(&basis.observation())
                    .map_err(|_| {
                        Stop::from("output correspondence requires an exact native observation")
                    })?;
                let work = branch
                    .exact_entity_state_read_work_bound()
                    .ok_or("output correspondence has no exact native probe")?;
                let navigation = branch
                    .exact_entity_state_read_navigation_work_bound()
                    .ok_or("output correspondence has no exact native navigation")?;
                let bound = runtime.history.catalog_len();
                let _scratch = reserve(
                    lease,
                    bound
                        .checked_mul(std::mem::size_of::<crate::history::data::CommitId>())
                        .ok_or_else(memory_overflow)?,
                )?;
                let mut visited = Vec::with_capacity(bound);
                let mut first: Option<Origin> = None;
                for (index, &(id, kind)) in roles.iter().enumerate() {
                    charge(
                        context,
                        index.saturating_add(1),
                    )?;
                    if roles[..index].iter().any(|(other, _)| *other == id) {
                        return Err(
                            "output correspondence requires distinct exact native identities".into()
                        );
                    }
                    context.checkpoint_cost(work, navigation).map_err(request)?;
                    let origin = branch
                        .with_exact_entity_state(id, |metadata, payload| {
                            // The native probe permits wildcard lookup; historical roles do not.
                            if metadata.entity_id != id || metadata.kind_id != kind {
                                return Err(Stop::from(
                                    "output correspondence native identity or kind differs",
                                ));
                            }
                            let payload =
                                payload.ok_or("output correspondence has no native payload")?;
                            for (_, value) in payload.aspects().entries() {
                                charge(context, value.semantic_byte_width())?;
                            }
                            original_association(
                                runtime,
                                basis,
                                id,
                                payload,
                                &mut visited,
                                bound,
                                context,
                            )
                        })
                        .map_err(|_| Stop::from("output correspondence native probe refused"))?
                        .ok_or("output correspondence native identity is absent or retired")??;
                    charge(
                        context,
                        origin
                            .producer
                            .len()
                            .saturating_add(producer.len())
                            .saturating_add(32),
                    )?;
                    if origin.producer != producer || &origin.dependency_identity != dependency {
                        return Err(
                            "output correspondence producer association differs from native publication"
                                .into(),
                        );
                    }
                    if let Some(first) = &first {
                        if !std::sync::Arc::ptr_eq(first, &origin) {
                            return Err(
                                "output correspondence spans different native publications"
                                    .into(),
                            );
                        }
                    } else {
                        charge(
                            context,
                            roles.len().saturating_mul(
                                1 + (usize::BITS - origin.outputs.len().leading_zeros()) as usize,
                            ),
                        )?;
                        if origin.outputs.len() != roles.len()
                            || roles
                                .iter()
                                .any(|(id, _)| origin.outputs.binary_search(id).is_err())
                        {
                            return Err(
                                "output correspondence omits or adds native producer output"
                                    .into(),
                            );
                        }
                        first = Some(origin);
                    }
                }
                Ok(())
            },
        )
    });
    execution.consult().map_err(request)?;
    result.map_err(request)?.map_err(request)?.0
}
fn original_association(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    id: EntityId,
    payload: &worth_foundational::facade::AuthoritativeRecordAspectState,
    visited: &mut Vec<crate::history::data::CommitId>,
    bound: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Origin, Stop> {
    visited.clear();
    let mut next = basis.inner.root.commit_id();
    loop {
        // Cold readmission: the actual catalog cardinality bounds its lookup;
        // visited.len() additionally pays the linear cycle check.
        charge(
            context,
            bound.saturating_add(visited.len()).saturating_add(1),
        )?;
        let commit = next.ok_or("output correspondence has no original producer history")?;
        if visited.len() >= bound || visited.contains(&commit) {
            return Err("output correspondence ancestry is incomplete or cyclic".into());
        }
        visited.push(commit);
        let envelope = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("output correspondence lost native history")?;
        require_linear_history(&envelope)?;
        charge(
            context,
            envelope
                .patch
                .authoritative_record_patches
                .len()
                .saturating_add(envelope.merged_plan.merged_intents.len()),
        )?;
        if envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| patch.target == RecordRef::Entity(id))
            || envelope
                .merged_plan
                .merged_intents
                .iter()
                .any(|intent| intent_writes_entity(intent, id))
        {
            envelope.readmit_required_output_payload(id, payload, context)?;
            return Ok(std::sync::Arc::clone(
                envelope
                    .required_output_origin
                    .as_ref()
                    .ok_or("output correspondence lost original association")?,
            ));
        }
        next = envelope.commit.parents.first().copied();
    }
}
