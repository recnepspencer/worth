//! A complete discovered tree can retain a verified root without minting a settlement.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryVerifiedHistoryParent;

pub(super) struct VerifiedHistoryOutputTree {
    pub(super) dependents: Vec<Arc<WorthQueryOutputDemandSettlement>>,
    dependencies: Option<WorthQueryRetainedQueryDependencies>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    /// The SDK supplies the actual installed discovery continuation's closed
    /// children. Its parent remains Query-issued verified read custody.
    pub fn retain_verified_history_output_tree<R: ApplicationOutputGraphShape<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQueryVerifiedHistoryParent<S, P,
            <<R::RootConnection as worth_query_declaration::facade::application_program::ApplicationConnectionShape<S>>::Binding as crate::domain_computation::primary_graph::WorthQueryApplicationRequiredOutputConnection<S>>::Demand>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        dependents: impl ExactSizeIterator<Item = Arc<WorthQueryOutputDemandSettlement>>,
        dependencies: &mut Option<WorthQueryRetainedQueryDependencies>,
    ) -> Result<Arc<WorthQueryRetainedHistoryOutputTree>, ReadDenial>
    where
        <R::RootConnection as worth_query_declaration::facade::application_program::ApplicationConnectionShape<S>>::Binding: crate::domain_computation::primary_graph::WorthQueryApplicationRequiredOutputConnection<S>,
{
        authority
            .validate(phase, self.runtime(), authority.request())
            .map_err(history)?;
        let custody = parent.custody();
        let profile = custody.profile();
        if !profile.same_custody(authority.profile())
            || !profile.matches_program(self)
            || profile.root != crate::equality_type_id::EqualityTypeId::of::<R>().into_external()
            || !Arc::ptr_eq(custody.original(), authority.original())
        {
            return Err(foreign());
        }
        if !profile.leaf && dependencies.is_none() {
            return Err(ReadDenial::Incomplete {
                model: profile.model.entity(),
            });
        }
        let execution = phase
            .checked_execution_request_for(self.runtime().product_runtime())
            .map_err(stopped)?;
        let count = dependents.len();
        let bytes = count
            .checked_mul(std::mem::size_of::<Arc<WorthQueryOutputDemandSettlement>>())
            .and_then(|bytes| {
                bytes.checked_add(
                    std::mem::size_of::<WorthQueryRetainedHistoryOutputTree>()
                        + 2 * std::mem::size_of::<usize>(),
                )
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| resource(Cause::ChargedBytesOverflow))?;
        let memory =
            memory::reserve(&execution, execution.has_accounting(), bytes).map_err(resource)?;
        let mut retained = Vec::with_capacity(count);
        for dependent in dependents {
            checkpoint(&execution)?;
            if retained.len() == count || !dependent.belongs_to(self.runtime()) {
                return Err(foreign());
            }
            retained.push(dependent);
        }
        if retained.len() != count {
            return Err(foreign());
        }
        let tree = Arc::new(WorthQueryRetainedHistoryOutputTree {
            profile: profile.clone(),
            source: Arc::clone(custody.original()),
            kind: HistoryRetainedOutputTree::Verified(VerifiedHistoryOutputTree {
                dependents: retained,
                dependencies: dependencies.take(),
                _memory: memory,
            }),
        });
        authority
            .confirm_read(phase, self.runtime(), authority.request(), tree)
            .map_err(history)
    }
}
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(super) fn qualify_verified_history_forest(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        trees: &[Arc<WorthQueryRetainedHistoryOutputTree>],
    ) -> Result<WorthQueryCompletedHistoryOutputs, ReadDenial> {
        let execution = phase
            .checked_execution_request_for(self.product_runtime())
            .map_err(stopped)?;
        for tree in trees {
            checkpoint(&execution)?;
            if !Arc::ptr_eq(&tree.source, authority.original()) {
                return Err(foreign());
            }
        }
        let settlements = trees.iter().flat_map(|tree| tree.settlements());
        let endpoint = crate::domain_computation::primary_graph::application_output_demand::original_endpoint::derive(
            &execution, authority.original(), &mut settlements.clone(), &mut settlements.clone(),
        ).map_err(Original::Output)?;
        for tree in trees {
            checkpoint(&execution)?;
            let selected = self
                .select_application_read_observation(phase, &endpoint)
                .map_err(Original::Observation)?;
            tree.require_current_at(selected, phase, access, authority.request())?;
            if matches!(&tree.kind, HistoryRetainedOutputTree::Verified(_)) {
                self.require_verified_root_at(
                    &execution,
                    authority,
                    &self
                        .select_application_read_observation(phase, &endpoint)
                        .map_err(Original::Observation)?,
                    &tree.profile,
                )?;
            }
        }
        let selected = self
            .select_application_read_observation(phase, &endpoint)
            .map_err(Original::Observation)?;
        super::super::super::output_coverage::require(
            &execution,
            &selected,
            authority,
            settlements,
        )?;
        self.require_skipped_history_roots(&execution, authority, &selected, trees)?;
        let owner = self
            .selected_program_owner(&selected)
            .map_err(Original::Program)?;
        for profile in authority.profile().tree_profiles() {
            checkpoint(&execution)?;
            if owner.owned_revision() != &profile.program
                || !owner
                    .owns_output_root(crate::equality_type_id::EqualityTypeId::from(profile.root))
            {
                return Err(foreign());
            }
        }
        Ok(WorthQueryCompletedHistoryOutputs {
            profile: authority.profile().clone(),
            original: HistoryCompletedOutputBasis::Verified {
                source: Arc::clone(authority.original()),
                endpoint,
            },
            forest: Some(retain_fallback_trees(&execution, trees)?),
        })
    }
    fn require_verified_root_at(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            S,
        >,
        profile: &WorthQueryHistoryOutputProfile,
    ) -> Result<(), ReadDenial> {
        self.runtime
            .primary_provider
            .graph
            .with_runtime(execution, |native| {
                let evidence = authority
                    .selected_output_read(&execution, native)
                    .map_err(ReadDenial::SelectionRead)?;
                if !super::super::super::output_verification::selected_profile_root_is_current(
                    execution, native, &evidence, selected, profile,
                )? {
                    return Err(ReadDenial::Incomplete {
                        model: evidence.model(),
                    });
                }
                Ok(())
            })
    }
}

impl WorthQueryRetainedHistoryOutputTree {
    pub(in crate::domain_computation::primary_graph) fn require_current_at<S: ApplicationSchema>(
        &self,
        selected: crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            S,
        >,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<(), ReadDenial> {
        match &self.kind {
            HistoryRetainedOutputTree::Original(original) => original
                .require_current_at(selected, phase, access, request)
                .map_err(ReadDenial::Original),
            HistoryRetainedOutputTree::Verified(verified) => {
                if let Some(dependencies) = &verified.dependencies {
                    selected
                        .require_current_required_set(
                            phase,
                            access,
                            request,
                            verified.dependents.iter().map(Arc::as_ref),
                            dependencies,
                        )
                        .map_err(Original::Currentness)?;
                } else {
                    selected
                        .require_current_output_settlements(
                            phase,
                            verified.dependents.iter().map(Arc::as_ref),
                        )
                        .map_err(Original::Output)?;
                }
                Ok(())
            }
        }
    }
}
