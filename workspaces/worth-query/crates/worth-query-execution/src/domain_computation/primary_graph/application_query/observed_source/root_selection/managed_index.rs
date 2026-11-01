//! Complete Source dependency custody, separate from callback permission.
use super::super::source_identity::WorthQueryObservedSourceMeaning;
use super::*;
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryInstalledManagedIndexAccessContract;

pub(in crate::domain_computation::primary_graph::application_query) struct ManagedSourceDependencies
{
    pub(in crate::domain_computation::primary_graph::application_query) authorization:
        Arc<crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization>,
    pub(in crate::domain_computation::primary_graph::application_query) rows:
        worth_execution::ExecutionArray<Arc<WorthQueryObservedSourceMeaning>>,
    pub(in crate::domain_computation::primary_graph::application_query) result_set:
        Option<Arc<WorthQueryObservedSourceMeaning>>,
    pub(in crate::domain_computation::primary_graph::application_query) maintained: Option<
        Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    >,
    pub(in crate::domain_computation::primary_graph::application_query) identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph::application_query) fact_count: usize,
    pub(in crate::domain_computation::primary_graph::application_query) field_count: usize,
    pub(in crate::domain_computation::primary_graph::application_query) _allocation:
        Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
}
impl ManagedSourceDependencies {
    pub(in crate::domain_computation::primary_graph::application_query) fn meanings(
        &self,
    ) -> impl Iterator<Item = &Arc<WorthQueryObservedSourceMeaning>> {
        self.rows.elements().iter().chain(self.result_set.iter())
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn accepts(
        meaning: &WorthQueryObservedSourceMeaning,
    ) -> bool {
        let source = meaning.footprint();
        source.complete
            && source.root_selection.as_ref().is_none_or(|selection| {
                selection.managed_index.is_none()
                    && selection.orbit.is_none()
                    && selection.components.is_none()
            })
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn len(&self) -> usize {
        if self.maintained.is_some() {
            return 2;
        }
        self.rows
            .len()
            .checked_add(2)
            .expect("admitted completed-source descriptor count")
    }
}
impl std::fmt::Debug for ManagedSourceDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedSourceDependencies")
            .field("rows", &self.rows.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph::application_query) struct DeclaredManagedIndexSelection
{
    pub(in crate::domain_computation::primary_graph::application_query) coverage: Option<
        Arc<crate::domain_computation::primary_graph::access_strategy::coverage::RetainedCoverage>,
    >,
    pub(in crate::domain_computation::primary_graph::application_query) access:
        Arc<WorthQueryInstalledManagedIndexAccessContract>,
    pub(in crate::domain_computation::primary_graph::application_query) dependencies:
        Arc<ManagedSourceDependencies>,
}
impl DeclaredManagedIndexSelection {
    fn coverage_is_present(&self) -> bool {
        self.access.meaning().definition().parts().membership ==
            worth_query_declaration::facade::application_query::ApplicationManagedIndexMembership::Candidates
            || self.coverage.is_some()
    }
    pub(in crate::domain_computation::primary_graph::application_query) const fn descriptor_work(
    ) -> u64 {
        b"declared-managed-index:v1".len() as u64 + 64
    }
    pub(super) fn append_descriptor(&self, digest: &mut Sha256) {
        digest.update(b"declared-managed-index:v1");
        digest.update(self.access.graph_obligations().identity().bytes());
        digest.update(self.dependencies.identity);
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn same_truth(
        &self,
        other: &Self,
        walks: &mut crate::domain_computation::primary_graph::source_read_equivalence::SourceReadEquivalence,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: &dyn Fn() -> Result<
            (),
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        >,
        admit: &mut impl FnMut(
            u64,
        ) -> Result<
            (),
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        >,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        control()?;
        if !self.coverage_is_present() || !other.coverage_is_present() {
            return Ok(false);
        }
        if !self
            .dependencies
            .authorization
            .same_truth(&other.dependencies.authorization, admit)?
        {
            return Ok(false);
        }
        match (
            &self.dependencies.maintained,
            &other.dependencies.maintained,
        ) {
            (None, None) => {}
            (Some(left), Some(right)) if left.same_truth(right, admit)? => {}
            _ => return Ok(false),
        }
        if self.access.graph_obligations().identity() != other.access.graph_obligations().identity()
            || self.dependencies.identity != other.dependencies.identity
            || self.dependencies.rows.len() != other.dependencies.rows.len()
        {
            return Ok(false);
        }
        for (left, right) in self
            .dependencies
            .meanings()
            .zip(other.dependencies.meanings())
        {
            control()?;
            super::super::source_identity::admit_footprint_comparison(
                left.footprint(),
                right.footprint(),
                &mut |units| {
                    admit(u64::try_from(units).map_err(|_| worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)?)
                },
            )?;
            if left.checkpoint_identity() != right.checkpoint_identity()
                || !super::super::comparison::equal(
                    left.footprint(),
                    right.footprint(),
                    walks,
                    execution,
                    control,
                )?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
impl PartialEq for DeclaredManagedIndexSelection {
    fn eq(&self, other: &Self) -> bool {
        self.coverage_is_present()
            && other.coverage_is_present()
            && self.access.graph_obligations().identity()
                == other.access.graph_obligations().identity()
            && self.dependencies.authorization == other.dependencies.authorization
            && self.dependencies.maintained == other.dependencies.maintained
            && self.dependencies.identity == other.dependencies.identity
            && self.dependencies.rows.len() == other.dependencies.rows.len()
            && self
                .dependencies
                .meanings()
                .zip(other.dependencies.meanings())
                .all(|(a, b)| {
                    a.checkpoint_identity() == b.checkpoint_identity()
                        && a.footprint() == b.footprint()
                })
    }
}
impl Eq for DeclaredManagedIndexSelection {}
