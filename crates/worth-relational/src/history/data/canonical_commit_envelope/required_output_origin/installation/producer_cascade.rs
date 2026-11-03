//! Native derives succession roots from actual deleted predecessor membership.
use super::{route_completion::RouteCompletion, *};
use crate::{
    history::data::{codec_retirement_closure::source_scope_closure, SelectedOutputPublication},
    identity::data::EntityId,
    mvcc::{RequiredOutputRolePosture as Posture, RequiredOutputRoleTarget},
    publication::{
        data::AuthoredRetirementRefusal as Cause, patch::data::RecordStructuralChange as Change,
    },
    transactions::data::{EntityReference, RecordRef},
};
use std::sync::Arc;
mod roster;

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CanonicalProducerRetirement {
    pub(crate) anchors: Vec<EntityId>,
    pub(crate) publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
}

pub(super) struct ProducerRetirementClosure {
    admission: worth_proof::AdmittedProducerOutputRetirement,
    generator: worth_proof::RequiredOutputPublicationAuthority,
    anchors: Vec<EntityId>,
    selected: Vec<SelectedOutputPublication>,
    root_roles: Vec<RequiredOutputRoleTarget>,
}

pub(super) fn reject(cause: Cause, id: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(
        crate::publication::data::PublicationError::authored_retirement(
            cause,
            RecordRef::Entity(id),
        ),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare(
    candidate: &CanonicalCommitEnvelope,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    prior: &CanonicalRequiredOutputOrigin,
    roles: &[RequiredOutputRoleTarget],
    succession: &worth_proof::AdmittedRequiredOutputSuccession,
    generator: &worth_proof::RequiredOutputPublicationAuthority,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<ProducerRetirementClosure>, TransactionCommitError> {
    let Some(removal) = scope_dependents::select_removed_owning_scopes(
        candidate, runtime, installed, prior, roles, context,
    )?
    else {
        return Ok(None);
    };
    let anchors = removal.anchors;
    let latest = source_scope_closure::current(
        candidate.commit.parents.first().copied(),
        runtime.history.canonical_catalog_len(),
        Some(installed),
        &mut |commit| runtime.history.canonical_envelope(commit),
        &mut |envelope, context| {
            source_scope_closure::validate_preparation_history(envelope, &runtime.history, context)
                .map_err(Into::into)
        },
        context,
    )
    .map_err(|failure| typed_corruption(relation_stop(failure.stop), anchors[0]))?;
    let selected =
        source_scope_closure::select(&anchors, &latest, context).map_err(relation_stop)?;
    if selected.is_empty() {
        return Err(reject(
            Cause::CorruptPublication,
            removal.dependent.meaning.source_scope,
        ));
    }
    // A legacy single-publication candidate supplying no dependent retirement
    // retains its original ownership refusal and exact publication/owner subjects.
    let mut supplies_dependent = false;
    for publication in &selected {
        context
            .checkpoint(
                roles
                    .len()
                    .saturating_mul(publication.origin.members.len() + 1) as u64,
            )
            .map_err(origin_request)?;
        supplies_dependent |= roles.iter().any(|role| matches!(role.target, EntityReference::Existing(id) if publication.origin.outputs.binary_search(&id).is_ok()));
    }
    if !supplies_dependent {
        super::super::source_scope_ownership::require_cascade(&removal.dependent, None)?;
    }
    for publication in &selected {
        context.checkpoint(1).map_err(origin_request)?;
        if publication
            .origin
            .issuer
            .as_ref()
            .is_some_and(|issuer| !issuer.same_publication_owner(&installed.authority))
        {
            return Err(reject(
                Cause::DifferentSourcePublication,
                publication.origin.meaning.source_scope,
            ));
        }
    }
    let root_roles = roster::require(candidate, prior, roles, &selected, &latest, context)?;
    context
        .checkpoint(selected.len().saturating_mul(selected.len() + 1) as u64)
        .map_err(origin_request)?;
    let authorities = selected
        .iter()
        .map(|selected| {
            installed.authority.seal_required_output_publication_at(
                selected.origin.commit.commit_id.0,
                selected.ordinal,
            )
        })
        .collect::<Vec<_>>();
    let admission = installed
        .authority
        .admit_source_scope_retirements(succession, authorities)
        .ok_or_else(|| reject(Cause::CorruptPublication, anchors[0]))?;
    Ok(Some(ProducerRetirementClosure {
        admission,
        generator: generator.clone(),
        anchors,
        selected,
        root_roles,
    }))
}

impl CanonicalCommitEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn install_producer_cascade(
        &mut self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        basis: &AdmittedRelationalBranchBasis,
        created: &CommitCreatedEntityBindings,
        created_relations: &CommitCreatedRelationBindings,
        state: &WorkingState,
        installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
        root: RouteCompletion<'_>,
        closure: ProducerRetirementClosure,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<crate::history::data::RequiredOutputPublicationPositions, TransactionCommitError>
    {
        let mut positions = self.install_route(
            runtime,
            basis,
            created,
            created_relations,
            state,
            installed,
            RouteCompletion {
                roles: &closure.root_roles,
                ..root
            },
            lease,
            context,
        )?;
        let root_origin = self
            .required_output_origin
            .take()
            .expect("installed generator route");
        let mut publications = Vec::with_capacity(closure.selected.len());
        let admission_work = closure.selected.len().saturating_add(1);
        for selected in closure.selected {
            let origin = &selected.origin;
            // Native constructs custody only for the exact canonical selection, never a caller roster.
            let predecessor = crate::mvcc::RelationalRequiredOutputPublication {
                runtime_instance_id: runtime.runtime_instance_id(),
                envelope: selected.envelope,
                origin: Arc::clone(origin),
                authority: installed.authority.seal_required_output_publication_at(
                    origin.commit.commit_id.0,
                    selected.ordinal,
                ),
                readmitted_at: Some(basis.clone()),
            };
            context
                .checkpoint(admission_work as u64)
                .map_err(origin_request)?;
            if !installed.authority.admits_producer_output_retirement(
                &closure.admission,
                &closure.generator,
                predecessor.authority(),
            ) {
                return Err(reject(
                    Cause::CorruptPublication,
                    origin.meaning.source_scope,
                ));
            }
            predecessor::validate(
                self,
                runtime,
                basis,
                &predecessor,
                &origin.meaning,
                lease,
                context,
            )
            .map_err(|failure| typed_corruption(failure, origin.meaning.source_scope))?;
            context
                .checkpoint(root.roles.len().saturating_mul(origin.members.len() + 1) as u64)
                .map_err(origin_request)?;
            for role in root.roles {
                context
                    .checkpoint((role.role.len() + role.entity_name.len()) as u64)
                    .map_err(origin_request)?;
            }
            let roles = root.roles.iter().filter(|role| matches!(role.target, EntityReference::Existing(id) if origin.outputs.binary_search(&id).is_ok())).cloned().collect::<Vec<_>>();
            self.install_route(
                runtime,
                basis,
                created,
                created_relations,
                state,
                installed,
                RouteCompletion {
                    producer: &origin.producer,
                    dependency_identity: origin.dependency_identity,
                    meaning: &origin.meaning,
                    roles: &roles,
                    previous: Some(origin),
                    completion: CanonicalOutputPublication::ProducerRetirement {
                        predecessor: origin.commit.commit_id,
                    },
                },
                lease,
                context,
            )
            .map_err(|failure| typed_corruption(failure, origin.meaning.source_scope))?;
            let retired = self
                .required_output_origin
                .take()
                .expect("installed retired route");
            positions.insert(
                &retired.meaning,
                crate::history::data::RequiredOutputRoutePosition::Published(self.commit.commit_id),
                lease,
                context,
            )?;
            publications.push(retired);
        }
        self.required_output_origin = Some(root_origin);
        self.producer_cascade_retirement = Some(Arc::new(CanonicalProducerRetirement {
            anchors: closure.anchors,
            publications,
        }));
        Ok(positions)
    }
}
fn typed_corruption(failure: TransactionCommitError, subject: EntityId) -> TransactionCommitError {
    match failure {
        TransactionCommitError::Publication { ref error, .. }
            if error.authored_retirement_refusal().is_none() =>
        {
            reject(Cause::CorruptPublication, subject)
        }
        other => other,
    }
}
