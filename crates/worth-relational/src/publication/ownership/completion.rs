//! The publication writer replaces its current custody in the unpublished commit.
use super::{record, CurrentPublicationRoster};
use crate::authority::mutation::current_records::{
    CompletedCurrentRecords as CompletedOwnership, Writer,
};
use crate::schema::current_publication_ownership::{CATALOG, EDGE, PUBLICATION};
use crate::{
    branch::{AdmittedRelationalBranchBasis, SelectedRelationalBranchState},
    history::data::{CanonicalCommitEnvelope, CommitId},
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
    candidate: &CanonicalCommitEnvelope,
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
                let mut witnesses = Vec::new();
                let claims = super::read::publication_claims(
                    execution,
                    basis,
                    row.entity,
                    row.value.origin.commit.commit_id,
                    writer.context(),
                )?;
                for link in &claims.links {
                    writer.checkpoint(1)?;
                    if (replaced(&row.value.origin.meaning)
                        && link.kind == storage.publication_link
                        && link.source == row.entity)
                        || ([storage.member_link, storage.edge_link].contains(&link.kind)
                            && link.target == row.entity)
                    {
                        if link.kind == storage.edge_link {
                            witnesses.push(link.source);
                        }
                        writer.delete_relation(link.id)?;
                    }
                }
                for witness in witnesses {
                    writer.delete_entity(witness)?;
                }
                if replaced(&row.value.origin.meaning) {
                    writer.delete_entity(row.entity)?;
                }
            }
        }
        routes.retain(|route| !replaced(&route.meaning));
        let has_new = candidate.output_origins().any(|origin| {
            origin.source_scope_owned
                && origin.meaning.source_scope == scope
                && origin
                    .members
                    .iter()
                    .any(|member| member.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
        });
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
        for (ordinal, origin) in candidate
            .output_origins()
            .enumerate()
            .filter(|(_, origin)| {
                origin.source_scope_owned
                    && origin.meaning.source_scope == scope
                    && origin.members.iter().any(|member| {
                        member.posture != crate::mvcc::RequiredOutputRolePosture::Retire
                    })
            })
        {
            let Some(catalog) = catalog else {
                return Err(super::corrupt(
                    origin.commit.commit_id,
                    RecordRef::Entity(scope),
                ));
            };
            let publication = origin.commit.commit_id;
            let payload = record::Publication {
                origin: std::sync::Arc::clone(origin),
                ordinal,
            };
            let bytes = writer.encode(&payload)?;
            use sha2::Digest;
            let digest = sha2::Sha256::digest(&bytes).into();
            let row = writer.create_encoded_entity(
                storage.publication_kind,
                PUBLICATION,
                publication,
                bytes,
                Some((storage.publication_link, catalog, publication)),
            )?;
            routes.push(record::Route {
                meaning: origin.meaning.clone(),
                publication,
                digest,
            });
            for member in origin
                .members
                .iter()
                .filter(|member| member.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
            {
                writer.create_link(storage.member_link, publication, member.id, row)?;
            }
            for edge in &origin.relations {
                writer.create_entity(
                    storage.edge_kind,
                    EDGE,
                    publication,
                    &record::Edge {
                        publication,
                        edge: edge.clone(),
                    },
                    Some((storage.edge_link, row, publication)),
                )?;
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
    Ok(writer.finish())
}
