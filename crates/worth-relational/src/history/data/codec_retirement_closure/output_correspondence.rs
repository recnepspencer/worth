//! Re-admit complete original output association on the recovered source state.
use super::{charge, request, CodecRetirementClosureStop as Stop};
use crate::history::data::canonical_commit_envelope::OutputOriginHistory;
use crate::{branch::AdmittedRelationalBranchBasis, runtime::RelationalRuntime};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
#[derive(Clone)]
pub(crate) struct SelectedOutputPublication {
    pub(crate) evidence: crate::publication::ownership::PublicationEvidence,
    pub(crate) origin: std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
    pub(crate) ordinal: usize,
}

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
) -> Result<SelectedOutputPublication, Stop> {
    let runtime = &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest::new(runtime, execution);
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                if basis.descriptor().runtime_instance_id() != runtime.runtime.runtime_instance_id()
                    || !basis.is_current()
                {
                    return Err(
                        "output correspondence requires the exact current recovered basis".into(),
                    );
                }
                if roles.is_empty() {
                    return Err("output correspondence correspondence is empty".into());
                }
                let branch = runtime.runtime.read_truth()
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
                let bound = runtime.runtime.history.catalog_len();
                let mut first: Option<(crate::history::data::canonical_commit_envelope::OwnedValidatedOutputOrigins, Origin)> = None;
                let mut origin_frames = OutputOriginHistory::default();
                let validated_history = &mut origin_frames;
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
                    let (selected, origin) = if role.posture == crate::mvcc::RequiredOutputRolePosture::Retire {
                        let selected = super::membership::latest(runtime, basis, id, validated_history, context, lease)?;
                        let origin = super::membership::validate_member(runtime, basis, &selected, id, None, validated_history, context, lease)?;
                        (selected, origin)
                    } else { branch
                        .read_exact_entity_state(execution, id, |metadata, payload| {
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
                            let selected = super::membership::latest(runtime, basis, id, validated_history, context, lease)?;
                            let origin = super::membership::validate_member(runtime, basis, &selected, id, payload, validated_history, context, lease)?;
                            Ok::<_, Stop>((selected, origin))
                        })
                        .map_err(|denial| match denial {
                            crate::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => Stop::Request(cause),
                            crate::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => Stop::from("output correspondence native probe requires an exact basis"),
                        })?
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
                    if origin.producer != producer || origin.dependency_identity.bytes() != dependency || &origin.meaning != meaning {
                        return Err(
                            "output correspondence producer association differs from native publication"
                                .into(),
                        );
                    }
                    if let Some((_, first)) = &first {
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
                        first = Some((selected, origin));
                    }
                }
                let (envelope, origin) = first.ok_or("complete output association absent")?;
                super::relations::validate_complete_publication(runtime, &basis.inner.root, &origin, validated_history, context, lease)?;
                charge(context, bound)?;
                let mut ordinal = None;
                for (index, candidate) in envelope.output_origins().enumerate() {
                    charge(context, 1)?;
                    if std::sync::Arc::ptr_eq(candidate, &origin) { ordinal = Some(index); break; }
                }
                let ordinal = ordinal.ok_or("selected publication lost its canonical route coordinate")?;
                Ok(SelectedOutputPublication { evidence:crate::publication::ownership::PublicationEvidence::Canonical(std::sync::Arc::clone(&envelope)), origin, ordinal })
            },
        )
    });
    execution.consult().map_err(request)?;
    result.map_err(request)?.map_err(request)?.0
}
