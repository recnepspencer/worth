//! Complete Source dependency custody, separate from callback permission.
use super::super::source_identity::WorthQueryObservedSourceMeaning;
use super::*;
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryInstalledManagedIndexAccessContract;

pub struct ManagedSourceDependencies
{
    pub authorization:
        Arc<crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization>,
    pub rows:
        worth_execution::ExecutionArray<Arc<WorthQueryObservedSourceMeaning>>,
    pub result_set:
        Option<Arc<WorthQueryObservedSourceMeaning>>,
    pub maintained: Option<
        Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    >,
    pub applied: Option<Arc<dyn crate::ports::AppliedManagedSourcePort>>,
    pub identity: [u8; 32],
    pub fact_count: usize,
    pub field_count: usize,
    pub _allocation:
        Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
}
impl ManagedSourceDependencies {
    pub fn maintained(
        &self,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    > {
        self.maintained
            .as_ref()
            .or_else(|| self.applied.as_ref().and_then(|source| source.published()))
    }

    pub fn meanings(
        &self,
    ) -> impl Iterator<Item = &Arc<WorthQueryObservedSourceMeaning>> {
        self.rows.elements().iter().chain(self.result_set.iter())
    }
    pub fn accepts(
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
    pub fn len(&self) -> usize {
        if self.maintained.is_some() || self.applied.is_some() {
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
pub struct DeclaredManagedIndexSelection
{
    pub coverage: Option<Arc<dyn std::any::Any + Send + Sync>>,
    pub access:
        Arc<WorthQueryInstalledManagedIndexAccessContract>,
    pub dependencies:
        Arc<ManagedSourceDependencies>,
}
impl DeclaredManagedIndexSelection {
    fn coverage_is_present(&self) -> bool {
        self.access.meaning().definition().parts().membership ==
            worth_query_declaration::facade::application_query::ApplicationManagedIndexMembership::Candidates
            || self.coverage.is_some()
    }
    pub const fn descriptor_work(
    ) -> u64 {
        b"declared-managed-index:v1".len() as u64 + 64
    }
    pub fn append_descriptor(&self, digest: &mut Sha256) {
        digest.update(b"declared-managed-index:v1");
        digest.update(self.access.graph_obligations().identity().bytes());
        digest.update(self.dependencies.identity);
    }
    /// Only a joined applied Source may bridge its acquisition descriptor to
    /// a positioned descriptor. Both sides retain authentic Native observations.
    pub fn same_published_source<
        E: 'static,
    >(
        &self,
        other: &Self,
        current_source: Option<
            &crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
        >,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        if !self.coverage_is_present()
            || !other.coverage_is_present()
            || (self.dependencies.applied.is_none() && other.dependencies.applied.is_none())
            || self.access.graph_obligations().identity()
                != other.access.graph_obligations().identity()
        {
            return Ok(false);
        }
        match (
            self.dependencies.maintained(),
            current_source.or_else(|| other.dependencies.maintained().map(Arc::as_ref)),
        ) {
            (Some(left), Some(right)) => left.same_published_source(right, admit),
            _ => Ok(false),
        }
    }
    pub fn same_truth(
        &self,
        other: &Self,
        current_source: Option<
            &crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
        >,
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
            self.dependencies.maintained(),
            other.dependencies.maintained(),
        ) {
            (None, None) => {
                // Until publication, only the very same admitted acquisition
                // has the same meaning. A hash or a matching roster is not proof.
                match (&self.dependencies.applied, &other.dependencies.applied) {
                    (None, None) => {}
                    (Some(left), Some(right)) if std::ptr::addr_eq(Arc::as_ptr(left), Arc::as_ptr(right)) => {}
                    _ => return Ok(false),
                }
            }
            (Some(left), Some(right))
                if self.same_published_source(other, current_source, admit)?
                    || left.same_truth(
                        right,
                        admit,
                        &mut |bytes| execution.reserve_retained_memory(bytes).map_err(Into::into),
                        &mut |cause| cause.into_native(),
                        &mut |left, right| {
                            left.same_content(execution, right)
                                .map_err(|cause| cause.into_execution_cause())
                        },
                    )? => {}
            _ => return Ok(false),
        }
        if self.access.graph_obligations().identity() != other.access.graph_obligations().identity()
            || (self.dependencies.maintained().is_none()
                && self.dependencies.identity != other.dependencies.identity)
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
            && self.dependencies.maintained() == other.dependencies.maintained()
            && match (&self.dependencies.applied, &other.dependencies.applied) {
                (None, None) => true,
                (Some(left), Some(right)) => std::ptr::addr_eq(Arc::as_ptr(left), Arc::as_ptr(right)),
                _ => false,
            }
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
