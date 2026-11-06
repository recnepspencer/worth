//! Original tree evidence is retained separately from collection qualification.
use super::*;
mod verified;
use crate::domain_computation::primary_graph::{
    application_output_demand::{qualify_output_collection, OriginalOutputTree},
    structural_walk::memory,
    WorthQueryHistoryOutputAuthority, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

type ReadDenial = super::WorthQueryHistoryOutputQualificationDenial;
type Original = WorthQueryOriginalReadOutputDenial;

/// Original complete tree custody; it supplies no currentness or permission.
pub struct WorthQueryRetainedHistoryOutputTree {
    profile: WorthQueryHistoryOutputProfile,
    source: Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
    kind: HistoryRetainedOutputTree,
}

pub(super) struct HistoryForestCompletion {
    _trees: Vec<Arc<WorthQueryRetainedHistoryOutputTree>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    /// Retain the complete SDK tree after its genuine root completion. Any
    /// refusal leaves the original move-only discovery dependencies in place.
    pub fn retain_original_history_output_tree<R: ApplicationOutputGraphShape<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        source: &WorthQueryHistoryOutputSource,
        root: Arc<WorthQueryOutputDemandSettlement>,
        dependents: impl ExactSizeIterator<Item = Arc<WorthQueryOutputDemandSettlement>>,
        dependencies: &mut Option<WorthQueryRetainedQueryDependencies>,
    ) -> Result<Arc<WorthQueryRetainedHistoryOutputTree>, ReadDenial> {
        if source.profile.is_output_forest()
            || source.profile.root
                != crate::equality_type_id::EqualityTypeId::of::<R>().into_external()
            || source.profile.program_type
                != crate::equality_type_id::EqualityTypeId::of::<P>().into_external()
            || source.profile.program != *self.owned_revision()
        {
            return Err(foreign());
        }
        source
            .root_completion
            .require_original(&root)
            .map_err(Original::Output)?;
        self.validate_selected_output_root::<R>(phase, access, &source.source)
            .map_err(Original::Program)?;
        let execution = phase
            .checked_execution_request_for(&self.runtime().runtime.product_runtime)
            .map_err(stopped)?;
        let original = OriginalOutputTree::retain(
            &execution,
            &source.source,
            root,
            dependents,
            dependencies,
            std::mem::size_of::<WorthQueryRetainedHistoryOutputTree>()
                + 2 * std::mem::size_of::<usize>(),
        )?;
        Ok(Arc::new(WorthQueryRetainedHistoryOutputTree {
            profile: source.profile.clone(),
            source: Arc::clone(&source.source),
            kind: HistoryRetainedOutputTree::Original(original),
        }))
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Verify every original tree through the existing currentness owner at
    /// the one endpoint derived from all authentic publication movements.
    pub fn qualify_original_history_output_forest(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        trees: &[Arc<WorthQueryRetainedHistoryOutputTree>],
    ) -> Result<WorthQueryCompletedHistoryOutputs, ReadDenial> {
        authority
            .validate(phase, self, authority.request())
            .map_err(history)?;
        let result = self.qualify_history_forest_in_frame(phase, access, authority, trees);
        authority
            .confirm_read(phase, self, authority.request(), result)
            .map_err(history)?
    }

    fn qualify_history_forest_in_frame(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        trees: &[Arc<WorthQueryRetainedHistoryOutputTree>],
    ) -> Result<WorthQueryCompletedHistoryOutputs, ReadDenial> {
        let profile = authority.profile();
        if !profile.is_output_forest() || profile.tree_profiles().len() < trees.len() {
            return Err(foreign());
        }
        let execution = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(stopped)?;
        if trees.is_empty() {
            return Err(ReadDenial::Incomplete {
                model: profile.model.entity(),
            });
        }
        require_fallback_subset(&execution, profile, trees)?;
        let mut has_verified = false;
        for tree in trees {
            checkpoint(&execution)?;
            has_verified |= matches!(tree.kind, HistoryRetainedOutputTree::Verified(_));
        }
        if has_verified {
            return self.qualify_verified_history_forest(phase, access, authority, trees);
        }
        let original = qualify_output_collection(
            self,
            phase,
            access,
            authority.request(),
            trees.iter().filter_map(|tree| match &tree.kind {
                HistoryRetainedOutputTree::Original(original) => Some(original),
                _ => None,
            }),
        )?;
        let selected = self
            .select_application_read_observation(phase, original.original_endpoint())
            .map_err(Original::Observation)?;
        super::super::output_coverage::require(
            &execution,
            &selected,
            authority,
            trees.iter().flat_map(|tree| tree.settlements()),
        )?;
        self.require_skipped_history_roots(&execution, authority, &selected, trees)?;
        let owner = self
            .selected_program_owner(&selected)
            .map_err(Original::Program)?;
        for tree in profile.tree_profiles() {
            checkpoint(&execution)?;
            if owner.owned_revision() != &tree.program
                || !owner.owns_output_root(crate::equality_type_id::EqualityTypeId::from(tree.root))
            {
                return Err(foreign());
            }
        }
        let retained = retain_fallback_trees(&execution, trees)?;
        Ok(WorthQueryCompletedHistoryOutputs {
            profile: profile.clone(),
            original: HistoryCompletedOutputBasis::Original(original),
            forest: Some(retained),
        })
    }

    fn require_skipped_history_roots(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            S,
        >,
        trees: &[Arc<WorthQueryRetainedHistoryOutputTree>],
    ) -> Result<(), ReadDenial> {
        self.runtime
            .primary_provider
            .graph
            .with_runtime(&execution, |native| {
                let evidence = authority
                    .selected_output_read(&execution, native)
                    .map_err(ReadDenial::SelectionRead)?;
                for registered in authority.profile().tree_profiles() {
                    let mut original = false;
                    for tree in trees {
                        checkpoint(execution)?;
                        original |= registered.same_custody(&tree.profile);
                    }
                    if !original
                        && !super::super::output_verification::profile_is_current(
                            execution, native, &evidence, &selected, registered,
                        )?
                    {
                        return Err(ReadDenial::Incomplete {
                            model: evidence.model(),
                        });
                    }
                }
                Ok(())
            })
    }
}

fn foreign() -> ReadDenial {
    ReadDenial::Original(Original::Output(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ForeignSettlement,
        "history output forest correspondence",
    )))
}
fn history(cause: WorthQueryHistoryNavigationDenial) -> ReadDenial {
    ReadDenial::Control(Box::new(cause))
}
fn stopped(
    cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
) -> ReadDenial {
    ReadDenial::Original(Original::Output(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
        "history output forest",
    )))
}
fn resource(cause: Cause) -> ReadDenial {
    ReadDenial::Original(Original::Output(
        crate::domain_computation::primary_graph::output_reuse::read_denial(cause),
    ))
}
fn checkpoint(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<(), ReadDenial> {
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)
        })
        .map_err(|cause| resource(cause.into()))?
        .0
        .map_err(|cause| resource(worth_execution::WorkCeilingDenial::Stopped(cause).into()))
}

fn require_fallback_subset(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    profile: &WorthQueryHistoryOutputProfile,
    trees: &[Arc<WorthQueryRetainedHistoryOutputTree>],
) -> Result<(), ReadDenial> {
    let mut registered = profile.tree_profiles().iter();
    for tree in trees {
        let mut matched = false;
        for candidate in registered.by_ref() {
            checkpoint(execution)?;
            if candidate.same_custody(&tree.profile) {
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(foreign());
        }
    }
    Ok(())
}

fn retain_fallback_trees(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    trees: &[Arc<WorthQueryRetainedHistoryOutputTree>],
) -> Result<HistoryForestCompletion, ReadDenial> {
    let bytes = trees
        .len()
        .checked_mul(std::mem::size_of::<Arc<WorthQueryRetainedHistoryOutputTree>>())
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<WorthQueryCompletedHistoryOutputs>())
        })
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| resource(Cause::ChargedBytesOverflow))?;
    let memory = memory::reserve(execution, execution.has_accounting(), bytes).map_err(resource)?;
    Ok(HistoryForestCompletion {
        _trees: trees.to_vec(),
        _memory: memory,
    })
}

enum HistoryRetainedOutputTree {
    Original(OriginalOutputTree),
    Verified(verified::VerifiedHistoryOutputTree),
}
impl WorthQueryRetainedHistoryOutputTree {
    pub(in crate::domain_computation::primary_graph) fn settlements(
        &self,
    ) -> impl Iterator<Item = &WorthQueryOutputDemandSettlement> + Clone {
        let original = match &self.kind {
            HistoryRetainedOutputTree::Original(tree) => Some(tree),
            _ => None,
        };
        let verified = match &self.kind {
            HistoryRetainedOutputTree::Verified(tree) => Some(tree),
            _ => None,
        };
        original
            .into_iter()
            .flat_map(OriginalOutputTree::settlements)
            .chain(
                verified
                    .into_iter()
                    .flat_map(|tree| tree.dependents.iter().map(Arc::as_ref)),
            )
    }
}
