//! Immutable Native original rows join material to the protected World cut.
use super::super::idempotency::snapshot_resolution::original_binding;
use super::*;
use worth_foundational::facade::WorldOriginalFate;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_relational::facade::runtime::RelationalRuntime;
use crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember as Member;

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
    let layout = provider.graph.layout_ref().provider_idempotency();
    for ((identity, member), material) in &catalog.material {
        request.consult().map_err(Denial::Execution)?;
        let original = originals
            .original_state(*identity)
            .ok_or(Denial::OriginalMismatch)?;
        if original.program_revision().bytes() != &material.metadata.program {
            return Err(Denial::OriginalMismatch);
        }
        let expected_commit: Option<&worth_relational::facade::history::RelationalCommitReceipt> =
            match (&material.body, original.fate()) {
                (MaterialBody::Abandoned, WorldOriginalFate::Released) => continue,
                (MaterialBody::Captured(_), WorldOriginalFate::Released) => continue,
                (MaterialBody::Captured(_), WorldOriginalFate::Open) => continue,
                // A crash after World publication (or recovered continuation) and
                // before `seal_correction_material` appends `Ready` leaves the
                // captured record unsealed. Bind it through the exact World-to-Native
                // join and immutable idempotency checks below; readers still fail closed.
                (MaterialBody::Captured(_), WorldOriginalFate::Continued)
                    if !originals
                        .original_observations()
                        .any(|published| published.identity() == original.reserved_commit()) =>
                {
                    continue
                }
                (
                    MaterialBody::Captured(_),
                    WorldOriginalFate::Published | WorldOriginalFate::Continued,
                ) => None,
                (MaterialBody::Ready { ready, .. }, WorldOriginalFate::Published) => {
                    Some(&ready.commit)
                }
                (MaterialBody::Expired { commit }, WorldOriginalFate::Published) => Some(commit),
                (
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
        let commit_id = match expected_commit {
            Some(commit) => {
                if !originals.original_observations().any(|published| {
                    published.identity() == original.reserved_commit()
                        && published.relational_commit() == Some(commit.commit_id)
                }) {
                    return Err(Denial::OriginalMismatch);
                }
                commit.commit_id
            }
            None => originals
                .original_observations()
                .find(|published| published.identity() == original.reserved_commit())
                .and_then(|published| published.relational_commit())
                .ok_or(Denial::OriginalMismatch)?,
        };
        let index = subjects
            .binary_search_by_key(&commit_id, |subject| subject.receipt().commit_id)
            .map_err(|_| Denial::NativeSourceUnavailable)?;
        let subject = &subjects[index];
        if let Some(commit) = expected_commit
            && subject.receipt() != commit
        {
            return Err(Denial::OriginalMismatch);
        }
        let effect = super::native_source::read_subject(request, native, &original, subject)?;
        let mut binding = None;
        for creation in effect.semantic_result().created_entities() {
            if creation.key.kind != layout.entity_kind
                || !creation
                    .key
                    .client_key
                    .starts_with(super::super::idempotency::ORIGINAL_CLIENT_KEY_PREFIX)
            {
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
        let observe = crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding::observe_recorded_intent;
        let row = observe(&binding.intent).map_err(|_| Denial::OriginalBindingUnavailable)?;
        // The commit's one ORIGINAL row is its group root's binding. A later
        // authored stage (public #368) shares that envelope key; its own
        // recorded intent must describe exactly the captured mutation input.
        let (intent, joined) = match member {
            Member::Primary => (row, binding.intent == material.metadata.intent),
            Member::AuthoredStage { .. } => (
                observe(&material.metadata.intent).map_err(|_| Denial::OriginalMismatch)?,
                row.envelope.is_some(),
            ),
        };
        if !joined
            || binding.key != material.metadata.key
            || intent.root != material.metadata.intent_identity
            || !intent.matches_mutation_input(
                &material.metadata.mutation,
                &material.metadata.input_identity,
            )
        {
            return Err(Denial::OriginalMismatch);
        }
    }
    super::pin_join::admit(request, originals, catalog, native, provider)
}
