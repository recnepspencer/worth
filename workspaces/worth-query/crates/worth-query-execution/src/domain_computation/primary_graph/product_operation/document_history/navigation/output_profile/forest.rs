//! Immutable original tree correspondence, never output or navigation authority.
use super::*;
use crate::domain_computation::primary_graph::structural_walk::memory;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(super) struct HistoryOutputForest {
    members: Vec<WorthQueryHistoryOutputProfile>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryHistoryOutputProfile {
    pub fn is_output_forest(&self) -> bool {
        self.forest.is_some()
    }

    pub(in crate::domain_computation::primary_graph) fn tree_profiles(&self) -> &[Self] {
        self.forest
            .as_ref()
            .map_or(std::slice::from_ref(self), |forest| &forest.members)
    }

    pub(super) fn same_forest_custody(&self, other: &Self) -> bool {
        match (&self.forest, &other.forest) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }

    pub(super) fn equivalent_forest(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        other: &Self,
    ) -> Result<bool, Cause> {
        let (Some(left), Some(right)) = (&self.forest, &other.forest) else {
            return Ok(false);
        };
        if Arc::ptr_eq(left, right) {
            return Ok(true);
        }
        if left.members.len() != right.members.len() {
            return Ok(false);
        }
        for (left, right) in left.members.iter().zip(&right.members) {
            checkpoint(execution, 1)?;
            if !left.equivalent(execution, right)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    /// Aggregate actual installed profiles without erasing their original roots
    /// or source demands. It carries no grant and creates no candidate work.
    pub fn bind_history_output_forest(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        profiles: &[WorthQueryHistoryOutputProfile],
    ) -> Result<WorthQueryHistoryOutputProfile, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let history = self
            .application()
            .installed_schema()
            .installed_history::<H>()
            .map_err(|_| Denial::IntentDrift)?;
        if history.definition().parts().model_domain != worth_query_declaration::facade::application_history::ApplicationHistoryModelDomain::OwnedModelGraph {
            return Err(Denial::RequiresSemanticCorrection(WorthQueryHistoryNavigationEligibilityDenial::ModelDomain));
        }
        let first = profiles.first().ok_or(Denial::IntentDrift)?;
        let execution = phase
            .checked_execution_request_for(&self.application().runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let resource = |cause| {
            Denial::Document(
                crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial::Recording(cause),
            )
        };
        for (index, profile) in profiles.iter().enumerate() {
            checkpoint(&execution, 1).map_err(resource)?;
            if profile.forest.is_some()
                || !profile.uses_owned_model_graph()
                || profile.model != self.model
                || profile.program != first.program
                || profile.program_type != first.program_type
            {
                return Err(Denial::IntentDrift);
            }
            for prior in &profiles[..index] {
                checkpoint(&execution, 1).map_err(resource)?;
                if profile.root == prior.root
                    && profile
                        .source_intent
                        .equivalent(&execution, &prior.source_intent)
                        .map_err(resource)?
                {
                    return Err(Denial::IntentDrift);
                }
            }
        }
        let bytes = profiles
            .len()
            .checked_mul(std::mem::size_of::<WorthQueryHistoryOutputProfile>())
            .and_then(|bytes| {
                bytes.checked_add(
                    std::mem::size_of::<HistoryOutputForest>() + 2 * std::mem::size_of::<usize>(),
                )
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| resource(Cause::ChargedBytesOverflow))?;
        let reservation =
            memory::reserve(&execution, execution.has_accounting(), bytes).map_err(resource)?;
        checkpoint(&execution, profiles.len() as u64).map_err(resource)?;
        let mut aggregate = first.clone();
        aggregate.forest = Some(Arc::new(HistoryOutputForest {
            members: profiles.to_vec(),
            _memory: reservation,
        }));
        Ok(aggregate)
    }
}

fn checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    work: u64,
) -> Result<(), Cause> {
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
        })
        .map_err(Cause::from)?
        .0
        .map_err(|stop| Cause::from(worth_execution::WorkCeilingDenial::Stopped(stop)))
}
