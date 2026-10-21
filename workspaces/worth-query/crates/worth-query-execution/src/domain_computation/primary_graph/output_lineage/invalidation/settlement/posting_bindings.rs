//! Versioned exact settlement membership in the one physical reverse index.

use super::*;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) enum PostingBindings
{
    Settlements(SettlementPostingBindings),
    QueryRead(super::super::query_coverage::QueryReadBinding),
}
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) struct SettlementPostingBindings
{
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) projection:
        Arc<PostingProjection>,
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) settlements:
        OrdSet<Arc<RecordedSettlementIdentity>>,
}
impl PostingBindings {
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn settlements(
        &self,
    ) -> Option<&SettlementPostingBindings> {
        match self {
            Self::Settlements(binding) => Some(binding),
            Self::QueryRead(_) => None,
        }
    }
}

pub(super) fn bind(
    state: &mut MarkState,
    projection: &Arc<PostingProjection>,
    identity: &Arc<RecordedSettlementIdentity>,
) -> Result<(), CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();

    let mut binding = if let Some(existing) = state.posting_bindings.get(&projection.id) {
        let existing = existing
            .settlements()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        if !Arc::ptr_eq(&existing.projection, projection) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        existing.clone()
    } else {
        install_physical(state, projection.id, &projection.ordinals)?;

        SettlementPostingBindings {
            projection: Arc::clone(projection),
            settlements: OrdSet::new(),
        }
    };

    binding.settlements.insert(Arc::clone(identity));

    state.posting_bindings.insert(
        projection.id,
        Arc::new(PostingBindings::Settlements(binding)),
    );
    Ok(())
}

pub(in super::super) fn install_physical(
    state: &mut MarkState,
    id: u64,
    keys: &im::OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
) -> Result<(), CompanionPreflightStop> {
    for key in keys.keys() {
        crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();

        let (stored, mut groups) = match state.postings.get_key_value(key) {
            Some((stored, groups)) => (Arc::clone(stored), groups.clone()),
            None => (Arc::clone(key), OrdSet::new()),
        };

        groups.insert(id);

        state.postings.insert(stored, groups);
        state.posting_count = state
            .posting_count
            .checked_add(1)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    }
    Ok(())
}
