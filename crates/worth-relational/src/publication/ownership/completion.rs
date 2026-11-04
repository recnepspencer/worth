//! The publication writer replaces its current custody in the unpublished commit.
use super::{checkpoint, record, request, CurrentPublicationRoster};
use crate::schema::current_publication_ownership::{CATALOG, EDGE, LINK, PUBLICATION};
use crate::{
    branch::{AdmittedRelationalBranchBasis, SelectedRelationalBranchState},
    history::data::{CanonicalCommitEnvelope, CommitId},
    identity::data::{EntityId, KindId, RelationId},
    runtime::RelationalPreparationRuntime,
    storage::overlay::{CurrentPartitionAccess, OverlayStateView, WorkingState},
    transactions::data::*,
    validation::engine::state_view::InvariantStateView,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};
mod apply;
mod fields;
use apply::Writer;

pub(crate) struct CompletedOwnership {
    pub(crate) effect: crate::authority::mutation::MutationEffect,
    pub(crate) funding: ExecutionMemoryReservation,
}

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
    let mut scopes = Vec::new();
    for origin in candidate
        .output_origins()
        .filter(|origin| origin.source_scope_owned)
    {
        writer.checkpoint(scopes.len() + 1)?;
        if !scopes.contains(&origin.meaning.source_scope) {
            scopes.push(origin.meaning.source_scope);
        }
    }
    for retired in &candidate.required_output_codec_retirements {
        writer.checkpoint(scopes.len() + roster.publications.len() + 1)?;
        if roster
            .publications
            .iter()
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
        let changed = |meaning: &crate::mvcc::RequiredOutputPublicationMeaning| {
            candidate
                .output_origins()
                .any(|origin| origin.source_scope_owned && origin.meaning.same_route(meaning))
                || candidate
                    .required_output_codec_retirements
                    .iter()
                    .any(|event| event.meaning.same_route(meaning))
        };
        for row in roster
            .publications
            .iter()
            .filter(|row| row.value.origin.meaning.source_scope == scope)
        {
            writer.checkpoint(
                candidate.output_origins().count()
                    + candidate.required_output_codec_retirements.len()
                    + 1,
            )?;
            if changed(&row.value.origin.meaning) {
                let mut witnesses = Vec::new();
                for link in &roster.links {
                    writer.checkpoint(1)?;
                    if (link.kind == storage.publication_link && link.source == row.entity)
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
                writer.delete_entity(row.entity)?;
            }
        }
        routes.retain(|route| !changed(&route.meaning));
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
                Some((storage.scope_link, scope)),
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
                Some((storage.publication_link, catalog)),
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
                    Some((storage.edge_link, row)),
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
