mod aspect_versions;
pub(crate) mod canonical_deltas;
mod disjoint_publication;
mod mutation_delta_dispatch;
pub(crate) mod pending_identities;
pub(crate) use disjoint_publication::compose as compose_disjoint_publication;
pub(crate) use worth_relational_contracts::internal_port::authority::mutation::adjacency_delta;
pub(crate) mod current_records;
pub(crate) mod deletion_publication;
mod delta_symbols;
mod effect;
mod effect_assembly;
mod effect_diagnostics;
mod effect_publication;
mod execution;
mod field_versions;
pub(crate) mod intents;
mod mutation_context;
pub(crate) mod outcomes;
pub(crate) mod owned_graph_selection;
mod patch_details;
mod record_changes;
pub(crate) mod retained_entity_selection;
mod round_denial;
mod stale_targets;
mod succession_record;
pub(crate) use succession_record::{restamp_succession_entity, succession_record};
mod workspace;

pub(crate) use canonical_deltas::{
    CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta, EvaluatedAspectBinding,
    FoundationalPatchFragment,
};
pub(crate) use effect::{AdjacencyDelta, AdjacencyDeltaKind, MutationEffect};
pub(crate) use execution::{
    apply_plan_to_working_state, apply_plan_with_publication_edges, MutationApplyOutcome,
};
pub(crate) use field_versions::visit_changed_fields;
pub(crate) use outcomes::RecordMutation;
pub(crate) use record_changes::apply_adjacency_deltas_for_request;
pub(crate) use round_denial::MutationApplyDenial;
pub(crate) use workspace::{
    BranchLocalDeleteAllowance, MutationPreparationTelemetry, MutationWorkspace,
};
