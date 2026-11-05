//! Reissue selected producer custody under fresh provenance and original record IDs.
use super::*;
use crate::{
    history::data::RequiredOutputPublicationPositions,
    mvcc::{
        RelationalHistoricalOutputSelection, RelationalInstalledApplicationPublicationAuthority,
    },
    storage::overlay::WorkingState,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};
mod absence;
mod header;
mod records;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalHistoricalOutputSelection {
    pub(crate) selected_basis: RelationalBranchBasisDescriptor,
    pub(crate) publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) absences: Vec<absence::CanonicalHistoricalOutputAbsence>,
}

pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelope,
    basis: &AdmittedRelationalBranchBasis,
    working: &WorkingState,
    installed: RelationalInstalledApplicationPublicationAuthority,
    admission: &worth_proof::AdmittedHistoricalOutputSelection,
    selections: Vec<RelationalHistoricalOutputSelection>,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
    if !matches!(
        envelope.authority_kind,
        super::super::CanonicalCommitAuthorityKind::OwnedGraphSelection(_)
    ) {
        return Err(crate::mvcc::origin_denial(
            "historical publication selection requires the Native graph selection constituent",
        ));
    }
    let first = selections
        .first()
        .ok_or_else(|| crate::mvcc::origin_denial("empty historical publication selection"))?;
    let selected_basis = first
        .selected
        .descriptor()
        .in_canonical_frame()
        .map_err(|_| {
            crate::mvcc::origin_denial("historical selection basis cannot enter canonical frame")
        })?;
    let mut positions = basis.inner.root.output_publication_positions.clone();
    let mut publications = Vec::new();
    let mut absences = Vec::new();
    for selection in &selections {
        let prior = &selection.publication;
        let original = &prior.origin;
        let refuse = || records::refuse(original.commit.commit_id, original.meaning.source_scope);
        installed.require_current_ownership_storage(basis, original.meaning.source_scope)?;
        installed.require_current_ownership_storage(
            &selection.selected,
            original.meaning.source_scope,
        )?;
        if selection.current.descriptor() != basis.descriptor()
            || selection.selected.descriptor() != first.selected.descriptor()
            || admission.publication_count() != selections.len()
            || !installed
                .authority
                .admits_historical_output_selection(admission, prior.authority())
            || !original.source_scope_owned
            || original
                .members
                .iter()
                .all(|m| m.posture == crate::mvcc::RequiredOutputRolePosture::Retire)
        {
            return Err(refuse());
        }
        if !selection.select_version {
            absences.push(absence::install(
                selection,
                basis,
                working,
                envelope.commit.commit_id,
                envelope.commit.version_id,
                &mut positions,
                lease,
                context,
            )?);
            continue;
        }
        for previous in &publications {
            let previous: &Arc<CanonicalRequiredOutputOrigin> = previous;
            context
                .checkpoint(
                    (original.meaning.family.len() + previous.meaning.family.len() + 65) as u64,
                )
                .map_err(origin_request)?;
            if previous.meaning.same_route(&original.meaning) {
                return Err(refuse());
            }
        }
        records::validate(
            selection,
            basis,
            working,
            envelope.commit.version_id,
            context,
        )?;
        let bytes = origin_retention_bytes(
            envelope,
            basis,
            original.producer.len(),
            original.members.len(),
            original.relations.len(),
        )?;
        context.checkpoint(bytes).map_err(origin_request)?;
        let funding = ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
            .map_err(|cause| origin_request(crate::execution::denial::lease_denial(cause)))?;
        let members = original
            .members
            .iter()
            .filter(|m| m.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
            .map(|m| CanonicalOutputMember {
                role: m.role.clone(),
                entity_name: m.entity_name.clone(),
                id: m.id,
                kind: m.kind,
                posture: crate::mvcc::RequiredOutputRolePosture::Preserve,
                birth: m.birth,
                birth_producer: m.birth_producer.clone(),
                birth_dependency: m.birth_dependency,
                before_write: records::entity_version(basis, m.id),
                after_write: envelope.commit.version_id,
            })
            .collect::<Vec<_>>();
        let relations = original
            .relations
            .iter()
            .map(|edge| CanonicalRequiredOutputRelation {
                id: edge.id,
                kind: edge.kind,
                source: edge.source,
                target: edge.target,
                birth: edge.birth,
                before_write: records::relation_version(basis, edge.id),
                after_write: envelope.commit.version_id,
            })
            .collect();
        let origin = Arc::new(CanonicalRequiredOutputOrigin {
            commit: envelope.commit.clone(),
            basis: basis
                .descriptor()
                .in_canonical_frame()
                .map_err(|_| refuse())?,
            producer: original.producer.clone(),
            source_scope_owned: true,
            dependency_identity: original.dependency_identity,
            outputs: members.iter().map(|m| m.id).collect(),
            members,
            relations,
            meaning: original.meaning.clone(),
            publication: CanonicalOutputPublication::HistoricalSelection {
                selected: original.commit.commit_id,
                generation: original.commit.version_id,
                predecessor: selection
                    .predecessor
                    .as_ref()
                    .map(|p| p.origin.commit.commit_id),
            },
            birth: original.birth,
            birth_producer: original.birth_producer.clone(),
            birth_dependency: original.birth_dependency,
            issuer: Some(installed.authority.clone()),
            _funding: Some(funding),
        });
        positions.insert(
            &origin.meaning,
            crate::history::data::RequiredOutputRoutePosition::Published(envelope.commit.commit_id),
            lease,
            context,
        )?;
        publications.push(origin);
    }
    envelope.historical_output_selection = Some(Arc::new(CanonicalHistoricalOutputSelection {
        selected_basis,
        publications,
        absences,
    }));
    envelope
        .validate_historical_selection_header(context)
        .map_err(|stop| match stop {
            crate::history::data::CodecRetirementClosureStop::Request(cause) => {
                origin_request(cause)
            }
            crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
                crate::mvcc::origin_denial(detail)
            }
        })?;
    Ok(positions)
}
