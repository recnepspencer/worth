//! Position transitions independently maintain presence; no roster row is consulted.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::group_selection::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::historical_selection::absence::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use super::*;
use crate::authority::mutation::current_records::CompletedCurrentRecords;
use crate::authority::mutation::current_records::Writer;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::config::data::StorageConfig;
use crate::history::data::CanonicalCommitEnvelopeBody;
use crate::schema::current_publication_ownership::POSITION;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionRequest, MapKernelContext};

#[allow(clippy::too_many_arguments)]
pub(crate) fn complete(
    execution: &ExecutionRequest<'_, '_>,
    storage_config: &StorageConfig,
    basis: &AdmittedRelationalBranchBasis,
    working: &mut WorkingState,
    identities: &mut crate::authority::mutation::pending_identities::PendingMutationIdentities,
    candidate: &CanonicalCommitEnvelopeBody,
    current: &[PresenceRow],
    positions: &mut Option<crate::history::data::RequiredOutputPublicationPositions>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CompletedCurrentRecords, TransactionCommitError> {
    let storage = (*basis
        .owner_inner()
        .root
        .schema_authority()
        .registry()
        .owner_current_publication_storage())
    .as_ref()
    .ok_or_else(|| {
        crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
            "position presence requires its installed storage",
        )
    })?;
    crate::publication::ownership::read_request::checkpoint(
        context,
        current
            .len()
            .saturating_mul(candidate.patch.authoritative_record_patches.len()) as u64
            + 1,
    )?;
    if positions.is_none()
        && current.iter().any(|(_, p)| {
            candidate
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == RecordRef::Entity(p.meaning.source_scope)
                        && patch.structural_change
                            == crate::publication::patch::data::RecordStructuralChange::Deleted
                })
        })
    {
        *positions = Some(
            basis
                .owner_inner()
                .root
                .output_publication_positions
                .clone(),
        );
    }
    let mut positions = positions.as_mut();
    let mut writer = Writer::in_stage(
        execution,
        storage_config,
        basis,
        working,
        identities,
        candidate,
        context,
    )?;
    for (ordinal, origin) in candidate
        .output_origins()
        .enumerate()
        .filter(|(_, o)| o.source_scope_owned)
    {
        writer.checkpoint(current.len() + 1)?;
        let old = current
            .iter()
            .find(|(_, p)| p.meaning.same_route(&origin.meaning));
        let live = origin
            .members
            .iter()
            .any(|m| m.posture != crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire);
        let next = if live {
            // Digest the canonical candidate, independently of the roster writer's effect.
            let bytes = writer.encode(&crate::publication::ownership::record::Publication {
                origin: std::sync::Arc::clone(origin),
                ordinal,
                custody: old.map(|(_, p)| p.custody()),
            })?;
            use sha2::Digest;
            Some(RoutePresence::published(
                origin,
                old.map(|(_, p)| p.custody()),
                sha2::Sha256::digest(bytes).into(),
            ))
        } else {
            None
        };
        if let Some(positions) = positions.as_deref_mut() {
            positions.current_owned.replace(
                execution,
                &origin.meaning,
                next.clone(),
                writer.context(),
            )?;
        }
        replace(&mut writer, storage, old, next)?;
    }
    let absence_work = candidate
        .historical_absent_routes()
        .fold(0usize, |work, event| {
            work.saturating_add(event.meaning.family.len() + 65)
        });
    if absence_work != 0 {
        writer.checkpoint(
            absence_work.saturating_mul(current.len().saturating_mul(3).saturating_add(1)),
        )?;
    }
    for event in candidate.historical_absent_routes() {
        let old = current
            .iter()
            .find(|(_, presence)| presence.meaning.same_route(&event.meaning));
        if let Some(positions) = positions.as_deref_mut() {
            positions
                .current_owned
                .replace(execution, &event.meaning, None, writer.context())?;
        }
        replace(&mut writer, storage, old, None)?;
    }
    for event in candidate.owner_required_output_codec_retirements() {
        writer.checkpoint(current.len() + 1)?;
        // A codec event for a non-opted-in route has no presence row.
        if let Some(old) = current
            .iter()
            .find(|(_, p)| p.meaning.same_route(&event.meaning))
        {
            let mut next = old.1.clone();
            next.posture = Posture::CodecRetired {
                retirement: candidate.commit.commit_id,
            };
            next.seal();
            if let Some(positions) = positions.as_deref_mut() {
                positions.current_owned.replace(
                    execution,
                    &event.meaning,
                    Some(next.clone()),
                    writer.context(),
                )?;
            }
            replace(&mut writer, storage, Some(old), Some(next))?;
        }
    }
    for old in current {
        writer.checkpoint(
            candidate.patch.authoritative_record_patches.len()
                + candidate.output_origins().count()
                + 1,
        )?;
        if !candidate
            .output_origins()
            .any(|origin| origin.meaning.same_route(&old.1.meaning))
            && !candidate
                .historical_absent_routes()
                .any(|event| event.meaning.same_route(&old.1.meaning))
            && candidate
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == RecordRef::Entity(old.1.meaning.source_scope)
                        && patch.structural_change
                            == crate::publication::patch::data::RecordStructuralChange::Deleted
                })
        {
            if let Some(positions) = positions.as_deref_mut() {
                positions.current_owned.replace(
                    execution,
                    &old.1.meaning,
                    None,
                    writer.context(),
                )?;
            }
            replace(&mut writer, storage, Some(old), None)?;
        }
    }
    Ok(writer.finish())
}
fn replace(
    writer: &mut Writer<'_, '_, '_, '_, '_>,
    storage: &crate::schema::data::CurrentPublicationStorage,
    old: Option<&PresenceRow>,
    next: Option<RoutePresence>,
) -> Result<(), TransactionCommitError> {
    if let Some((id, _)) = old {
        if let Some(next) = next {
            return writer.update_entity(*id, POSITION, next.publication, &next);
        }
        // The position owner owns exactly this private source link.
        let links = writer.current_links(*id, storage.position_link)?;
        for link in links {
            writer.delete_relation(link)?;
        }
        writer.delete_entity(*id)
    } else if let Some(next) = next {
        writer
            .create_entity(
                storage.position_kind,
                POSITION,
                next.publication,
                &next,
                Some((
                    storage.position_link,
                    next.meaning.source_scope,
                    CommitId(0),
                )),
            )
            .map(|_| ())
    } else {
        Ok(())
    }
}
