//! Re-admit complete original output association on the recovered source state.
use super::{charge, request, CodecRetirementClosureStop as Stop};
use crate::{branch::AdmittedRelationalBranchBasis, runtime::RelationalRuntime};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
type Origin =
    std::sync::Arc<super::super::canonical_commit_envelope::CanonicalRequiredOutputOrigin>;
pub(crate) fn validate_codec_output_correspondence(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    producer: &str,
    dependency: &[u8; 32],
    meaning: &crate::mvcc::RequiredOutputPublicationMeaning,
    roles: &[crate::mvcc::RequiredOutputRoleTarget],
) -> Result<std::sync::Arc<crate::history::data::CanonicalCommitEnvelope>, Stop> {
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
                let mut first: Option<Origin> = None;
                for (index, role) in roles.iter().enumerate() {
                    let crate::transactions::data::EntityReference::Existing(id) = role.target else {
                        return Err("recovered output role requires exact existing identity".into());
                    };
                    let kind = role.kind;
                    charge(
                        context,
                        index.saturating_add(1),
                    )?;
                    if roles[..index].iter().any(|other| other.target == crate::transactions::data::EntityReference::Existing(id)) {
                        return Err(
                            "output correspondence requires distinct exact native identities".into()
                        );
                    }
                    context.checkpoint_cost(work, navigation).map_err(request)?;
                    let origin = if role.posture == crate::mvcc::RequiredOutputRolePosture::Retire {
                        let selected = super::membership::latest(runtime, basis, id, context, lease)?;
                        super::membership::validate_member(runtime, basis, &selected, id, None, context, lease)?
                    } else { branch
                        .with_exact_entity_state(id, |metadata, payload| {
                            // The native probe permits wildcard lookup; historical roles do not.
                            if metadata.entity_id != id || metadata.kind_id != kind {
                                return Err(Stop::from(
                                    "output correspondence native identity or kind differs",
                                ));
                            }
                            if let Some(payload) = payload {
                                for (_, value) in payload.aspects().entries() {
                                    charge(context, value.semantic_byte_width())?;
                                }
                            }
                            let selected = super::membership::latest(runtime, basis, id, context, lease)?;
                            super::membership::validate_member(runtime, basis, &selected, id, payload, context, lease)
                        })
                        .map_err(|_| Stop::from("output correspondence native probe refused"))?
                        .ok_or("output correspondence native identity is absent or retired")?? };
                    charge(
                        context,
                        origin
                            .producer
                            .len()
                            .saturating_add(producer.len())
                            .saturating_add(meaning.family.len())
                            .saturating_add(origin.meaning.family.len())
                            .saturating_add(144),
                    )?;
                    charge(context, role.role.len().saturating_add(role.entity_name.len()).saturating_add(origin.members.len()))?;
                    if !origin.members.iter().any(|member| member.id == id && member.kind == kind && member.posture == role.posture
                        && member.role == role.role && member.entity_name == role.entity_name) {
                        return Err("output role differs from complete performed native correspondence".into());
                    }
                    if origin.producer != producer || &origin.dependency_identity != dependency || &origin.meaning != meaning {
                        return Err(
                            "output correspondence producer association differs from native publication"
                                .into(),
                        );
                    }
                    if let Some(first) = &first {
                        if first.commit != origin.commit {
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
                                .any(|role| match role.target { crate::transactions::data::EntityReference::Existing(id) => origin.outputs.binary_search(&id).is_err(), _ => true })
                        {
                            return Err(
                                "output correspondence omits or adds native producer output"
                                    .into(),
                            );
                        }
                        first = Some(origin);
                    }
                }
                let origin = first.ok_or("complete output association absent")?;
                super::relations::validate_complete_publication(runtime, &basis.inner.root, &origin, context, lease)?;
                charge(context, bound)?;
                runtime.history.canonical_envelope(origin.commit.commit_id).ok_or_else(|| Stop::from("complete publication history absent"))
            },
        )
    });
    execution.consult().map_err(request)?;
    result.map_err(request)?.map_err(request)?.0
}
