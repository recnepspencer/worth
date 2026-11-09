//! Advance current custody while preserving every unchanged claim row.
mod claims_delta;
mod successors;
use super::{record, CurrentPublicationRoster};
use crate::authority::mutation::current_records::{
    CompletedCurrentRecords as CompletedOwnership, Writer,
};
use crate::schema::current_publication_ownership::{CATALOG, PUBLICATION};
use crate::{
    branch::{AdmittedRelationalBranchBasis, SelectedRelationalBranchState},
    history::data::{CanonicalCommitEnvelopeBody, CommitId},
    runtime::RelationalPreparationRuntime,
    storage::overlay::{OverlayStateView, WorkingState},
    transactions::data::*,
    validation::engine::state_view::InvariantStateView,
};
use worth_execution::{ExecutionRequest, MapKernelContext};

pub(crate) fn complete(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    selected: &SelectedRelationalBranchState,
    working: &mut WorkingState,
    identities: &mut crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
    candidate: &CanonicalCommitEnvelopeBody,
    roster: &CurrentPublicationRoster,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CompletedOwnership, TransactionCommitError> {
    let Some(storage) = &basis
        .inner
        .root
        .schema_authority()
        .registry()
        .current_publication_storage
    else {
        return Err(crate::mvcc::origin_denial(
            "current ownership completion requires its Native storage grammar",
        ));
    };
    let mut writer = Writer::new(
        execution, runtime, basis, working, identities, candidate, context,
    )?;
    let absent_count = candidate.historical_absent_routes().count();
    if absent_count != 0 {
        let text_work = candidate
            .historical_absent_routes()
            .fold(0usize, |work, a| {
                work.saturating_add(a.meaning.family.len() + 65)
            });
        let rows = roster
            .publications
            .iter()
            .chain(&roster.codec_retired)
            .fold(roster.links.len() + 1, |rows, row| {
                rows.saturating_add(row.value.origin.members.len().saturating_mul(2))
                    .saturating_add(row.value.origin.relations.len().saturating_mul(3))
                    .saturating_add(4)
            });
        let scopes = candidate
            .output_origins()
            .count()
            .saturating_add(roster.codec_retired.len())
            .saturating_add(candidate.required_output_codec_retirements.len())
            .saturating_add(absent_count)
            .saturating_add(1);
        writer.checkpoint(
            text_work
                .saturating_mul(rows)
                .saturating_mul(scopes)
                .saturating_mul(4),
        )?;
        writer
            .retain((absent_count as u64).saturating_mul(
                2 * std::mem::size_of::<crate::identity::data::EntityId>() as u64,
            ))?;
    }
    let mut scopes = Vec::new();
    for absence in candidate.historical_absent_routes() {
        writer.checkpoint(scopes.len() + 1)?;
        if !scopes.contains(&absence.meaning.source_scope) {
            scopes.push(absence.meaning.source_scope);
        }
    }
    for origin in candidate
        .output_origins()
        .filter(|origin| origin.source_scope_owned)
    {
        writer.checkpoint(scopes.len() + 1)?;
        if !scopes.contains(&origin.meaning.source_scope) {
            scopes.push(origin.meaning.source_scope);
        }
    }
    for row in &roster.codec_retired {
        writer.checkpoint(candidate.patch.authoritative_record_patches.len() + 1)?;
        let scope = row.value.origin.meaning.source_scope;
        if candidate
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| {
                patch.target == RecordRef::Entity(scope)
                    && patch.structural_change
                        == crate::publication::patch::data::RecordStructuralChange::Deleted
            })
            && !scopes.contains(&scope)
        {
            scopes.push(scope);
        }
    }
    for retired in &candidate.required_output_codec_retirements {
        writer.checkpoint(
            scopes.len() + roster.publications.len() + roster.codec_retired.len() + 1,
        )?;
        if roster
            .publications
            .iter()
            .chain(&roster.codec_retired)
            .any(|row| row.value.origin.meaning.same_route(&retired.meaning))
            && !scopes.contains(&retired.meaning.source_scope)
        {
            scopes.push(retired.meaning.source_scope);
        }
    }
    for scope in scopes {
        let old_catalog = roster
            .catalogs
            .iter()
            .find(|(_, catalog)| catalog.scope == scope);
        writer.retain(roster._funding.bytes())?;
        let mut routes = old_catalog.map_or_else(Vec::new, |(_, catalog)| catalog.routes.clone());
        let scope_deleted = candidate
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| {
                patch.target == RecordRef::Entity(scope)
                    && patch.structural_change
                        == crate::publication::patch::data::RecordStructuralChange::Deleted
            });
        let successors = successors::select(candidate, scope, scope_deleted, &mut writer)?;
        let replaced = |meaning: &crate::mvcc::RequiredOutputPublicationMeaning| {
            scope_deleted
                || candidate
                    .historical_absent_routes()
                    .any(|absence| absence.meaning.same_route(meaning))
                || candidate
                    .output_origins()
                    .any(|origin| origin.source_scope_owned && origin.meaning.same_route(meaning))
        };
        let changed = |meaning: &crate::mvcc::RequiredOutputPublicationMeaning| {
            replaced(meaning)
                || candidate
                    .required_output_codec_retirements
                    .iter()
                    .any(|event| event.meaning.same_route(meaning))
        };
        for row in roster
            .publications
            .iter()
            .chain(&roster.codec_retired)
            .filter(|row| row.value.origin.meaning.source_scope == scope)
        {
            writer.checkpoint(
                candidate.output_origins().count()
                    + candidate.required_output_codec_retirements.len()
                    + 1,
            )?;
            if changed(&row.value.origin.meaning) {
                if !successors
                    .iter()
                    .any(|(_, origin)| origin.meaning.same_route(&row.value.origin.meaning))
                {
                    claims_delta::retire(
                        execution,
                        basis,
                        &mut writer,
                        storage,
                        row,
                        replaced(&row.value.origin.meaning),
                    )?;
                }
            }
        }
        routes.retain(|route| !replaced(&route.meaning));
        let has_new = !successors.is_empty();
        let catalog = if let Some((id, _)) = old_catalog {
            Some(*id)
        } else if has_new {
            let payload = record::Catalog {
                scope,
                routes: Vec::new(),
            };
            Some(writer.create_entity(
                storage.catalog_kind,
                CATALOG,
                CommitId(0),
                &payload,
                Some((storage.scope_link, scope, CommitId(0))),
            )?)
        } else {
            None
        };
        for (ordinal, origin) in successors {
            let Some(catalog) = catalog else {
                return Err(super::corrupt(
                    origin.commit.commit_id,
                    RecordRef::Entity(scope),
                ));
            };
            let publication = origin.commit.commit_id;
            let previous = roster
                .publications
                .iter()
                .chain(&roster.codec_retired)
                .find(|row| row.value.origin.meaning.same_route(&origin.meaning));
            let custody = previous.map(|row| row.value.custody());
            let payload = record::Publication {
                origin: std::sync::Arc::clone(origin),
                ordinal,
                custody,
            };
            let bytes = writer.encode(&payload)?;
            use sha2::Digest;
            let digest = sha2::Sha256::digest(&bytes).into();
            let row = if let Some(previous) = previous {
                writer.update_encoded_entity(
                    previous.entity,
                    PUBLICATION,
                    payload.custody(),
                    bytes,
                )?;
                claims_delta::advance(execution, basis, &mut writer, storage, previous, origin)?;
                previous.entity
            } else {
                writer.create_encoded_entity(
                    storage.publication_kind,
                    PUBLICATION,
                    publication,
                    bytes,
                    Some((storage.publication_link, catalog, publication)),
                )?
            };
            routes.push(record::Route {
                meaning: origin.meaning.clone(),
                publication,
                digest,
            });
            if previous.is_none() {
                claims_delta::insert(&mut writer, storage, row, publication, origin)?;
            }
        }
        if let Some(catalog) = catalog {
            let overlay = OverlayStateView::new(selected, writer.working);
            let after = InvariantStateView::new(&overlay, candidate.commit.version_id);
            if after.entity_metadata(scope).is_none() {
                if !routes.is_empty() {
                    return Err(super::corrupt(
                        routes[0].publication,
                        RecordRef::Entity(scope),
                    ));
                }
                for link in &roster.links {
                    writer.checkpoint(1)?;
                    if link.kind == storage.scope_link && link.source == catalog {
                        writer.delete_relation(link.id)?;
                    }
                }
                writer.delete_entity(catalog)?;
            } else {
                writer.update_entity(
                    catalog,
                    CATALOG,
                    CommitId(0),
                    &record::Catalog { scope, routes },
                )?;
            }
        }
    }
    let completed = writer.finish();
    crate::execution::phase_observation::ownership_rows(
        completed.effect.publication.changed_records.len(),
    );
    Ok(completed)
}
