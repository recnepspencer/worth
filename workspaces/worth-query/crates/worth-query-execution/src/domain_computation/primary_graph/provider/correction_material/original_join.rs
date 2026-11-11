//! Immutable Native original rows join material to the protected World cut.
use super::super::idempotency::snapshot_resolution::original_binding;
use super::*;
use worth_foundational::facade::WorldOriginalFate;
use worth_relational::facade::runtime::RelationalRuntime;

pub(super) fn admit(
    request: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    provider: &WorthQueryPrimaryGraphProvider,
    originals: &WorldProgramActivationHistory,
    catalog: &MaterialCatalog,
) -> Result<(), Denial> {
    if catalog.material.is_empty() {
        return Ok(());
    }
    let authority = native.durability_authority();
    let mut subjects = authority
        .discover_all_associated_effects(request)
        .map_err(super::native_source::denial)?;
    subjects.sort_unstable_by_key(|subject| subject.receipt().commit_id);
    if subjects
        .windows(2)
        .any(|pair| pair[0].receipt().commit_id == pair[1].receipt().commit_id)
    {
        return Err(Denial::OriginalMismatch);
    }
    let layout = provider.graph.layout.provider_idempotency();
    for (identity, material) in &catalog.material {
        request.consult().map_err(Denial::Execution)?;
        let original = originals
            .original_state(*identity)
            .ok_or(Denial::OriginalMismatch)?;
        if original.program_revision().bytes() != &material.metadata.program {
            return Err(Denial::OriginalMismatch);
        }
        let commit = match (&material.body, original.fate()) {
            (MaterialBody::Abandoned, WorldOriginalFate::Released) => continue,
            (MaterialBody::Captured(_), WorldOriginalFate::Released) => continue,
            (MaterialBody::Ready { ready, .. }, WorldOriginalFate::Published) => &ready.commit,
            (MaterialBody::Expired { commit }, WorldOriginalFate::Published) => commit,
            (
                MaterialBody::Captured(_),
                WorldOriginalFate::Published
                | WorldOriginalFate::Open
                | WorldOriginalFate::Continued,
            )
            | (
                MaterialBody::Ready { .. } | MaterialBody::Expired { .. },
                WorldOriginalFate::Open
                | WorldOriginalFate::Continued
                | WorldOriginalFate::Released,
            )
            | (
                MaterialBody::Abandoned,
                WorldOriginalFate::Published
                | WorldOriginalFate::Open
                | WorldOriginalFate::Continued,
            ) => return Err(Denial::OriginalMismatch),
        };
        if !originals.original_observations().any(|published| {
            published.identity() == original.reserved_commit()
                && published.relational_commit() == commit.commit_id
        }) {
            return Err(Denial::OriginalMismatch);
        }
        let index = subjects
            .binary_search_by_key(&commit.commit_id, |subject| subject.receipt().commit_id)
            .map_err(|_| Denial::NativeSourceUnavailable)?;
        let subject = &subjects[index];
        if subject.receipt() != commit {
            return Err(Denial::OriginalMismatch);
        }
        let effect = super::native_source::read_subject(request, native, &original, subject)?;
        let mut binding = None;
        for creation in effect.semantic_result().created_entities() {
            if creation.key.kind != layout.entity_kind {
                continue;
            }
            if binding.is_some() {
                return Err(Denial::OriginalBindingUnavailable);
            }
            let row = effect
                .created_entity(request, creation.assigned)
                .map_err(super::native_source::denial)?
                .ok_or(Denial::OriginalBindingUnavailable)?;
            binding = Some(
                original_binding::read(&row, layout)
                    .map_err(|_| Denial::OriginalBindingUnavailable)?,
            );
        }
        let binding = binding.ok_or(Denial::OriginalBindingUnavailable)?;
        let intent = crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding::observe_recorded_intent(&binding.intent)
            .map_err(|_| Denial::OriginalBindingUnavailable)?;
        if intent.root != material.metadata.intent_identity
            || !intent.matches_mutation_input(
                &material.metadata.mutation,
                &material.metadata.input_identity,
            )
        {
            return Err(Denial::OriginalMismatch);
        }
        if binding.key != material.metadata.key || binding.intent != material.metadata.intent {
            return Err(Denial::OriginalMismatch);
        }
    }
    super::pin_join::admit(request, originals, catalog, native, provider)
}
