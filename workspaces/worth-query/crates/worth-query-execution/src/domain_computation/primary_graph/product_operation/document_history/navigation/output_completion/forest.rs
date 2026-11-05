//! Original tree evidence is retained separately from collection qualification.
use super::*;
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
    original: OriginalOutputTree,
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
            original,
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
        if !profile.is_output_forest() || profile.tree_profiles().len() != trees.len() {
            return Err(foreign());
        }
        let execution = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(stopped)?;
        for (registered, tree) in profile.tree_profiles().iter().zip(trees) {
            checkpoint(&execution)?;
            if !registered.same_custody(&tree.profile) {
                return Err(foreign());
            }
        }
        super::super::output_coverage::require(
            &execution,
            self,
            authority,
            trees.iter().flat_map(|tree| tree.original.settlements()),
        )?;
        let original = qualify_output_collection(
            self,
            phase,
            access,
            authority.request(),
            trees.iter().map(|tree| &tree.original),
        )?;
        let selected = self
            .select_application_read_observation(phase, original.original_endpoint())
            .map_err(Original::Observation)?;
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
        let bytes = trees
            .len()
            .checked_mul(std::mem::size_of::<Arc<WorthQueryRetainedHistoryOutputTree>>())
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<WorthQueryCompletedHistoryOutputs>())
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| resource(Cause::ChargedBytesOverflow))?;
        let memory =
            memory::reserve(&execution, execution.has_accounting(), bytes).map_err(resource)?;
        Ok(WorthQueryCompletedHistoryOutputs {
            profile: profile.clone(),
            original,
            forest: Some(HistoryForestCompletion {
                _trees: trees.to_vec(),
                _memory: memory,
            }),
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
