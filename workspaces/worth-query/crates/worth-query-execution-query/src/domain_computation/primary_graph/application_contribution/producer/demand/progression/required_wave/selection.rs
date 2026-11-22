use super::*;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

/// A caller's Ready or an exactly requested upstream Ready anchors the wave.
/// `shared` and `positioned` retain its admitted Product/native proof basis.
pub struct RequiredWaveSelection<'runtime, Schema> {
    pub shared: SharedSelectedProductOperation<'runtime, Schema>,
    pub positioned: PositionedRelationalSnapshot,
    pub anchor_ready: SelectedReadyReadmission,
    pub branch: WorthQueryProductBranch,
    pub target: RequiredWaveTarget,
    state: SelectionState,
}

/// Delivery follows the actual successor custody; no unrelated entry thaws.
enum SelectionState {
    Live(crate::domain_computation::primary_graph::application_output_demand::LivePublicationHead),
    Frozen,
}
/// Only the selection owner can grant the live input to reselection.
struct LiveSelection<'runtime, Schema>(RequiredWaveSelection<'runtime, Schema>);

impl<'runtime, Schema: ApplicationSchema + 'static> RequiredWaveSelection<'runtime, Schema> {
    pub fn freeze(&mut self) {
        self.state = SelectionState::Frozen;
    }
    pub fn is_frozen(&self) -> bool {
        matches!(self.state, SelectionState::Frozen)
    }
    pub fn reselect(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        match self.state {
            SelectionState::Frozen => Ok(self),
            SelectionState::Live(_) => {
                reselect_required_wave(execution, runtime, LiveSelection(self))
            }
        }
    }
    /// Only actual custody delivery can thaw; unrelated entries stay frozen.
    pub fn after_awaited_delivery(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        _delivery: super::super::super::required_continuations::DeliveredAwaited,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let head = runtime.runtime.output_demands.publication_head_selection(
            self.shared
                .selected()
                .product()
                .observation()
                .lifecycle_incarnation(),
        )?;
        match head {
            crate::domain_computation::primary_graph::application_output_demand::PublicationHeadSelection::Frozen => Ok(self),
            crate::domain_computation::primary_graph::application_output_demand::PublicationHeadSelection::Live(_) =>
                select_required_wave_from_ready(execution, runtime, self.anchor_ready, self.branch, self.target),
        }
    }
    pub fn from_basis(
        shared: SharedSelectedProductOperation<'runtime, Schema>,
        positioned: PositionedRelationalSnapshot,
        anchor_ready: SelectedReadyReadmission,
        branch: WorthQueryProductBranch,
        target: RequiredWaveTarget,
        head: crate::domain_computation::primary_graph::application_output_demand::PublicationHeadSelection,
    ) -> Self {
        use crate::domain_computation::primary_graph::application_output_demand::PublicationHeadSelection;
        let state = match head {
            PublicationHeadSelection::Live(witness) => SelectionState::Live(witness),
            PublicationHeadSelection::Frozen => SelectionState::Frozen,
        };
        Self {
            shared,
            positioned,
            anchor_ready,
            branch,
            target,
            state,
        }
    }
}

pub fn select_required_wave<'runtime, Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    interest: &WorthQueryOutputDemandInterest,
    branch: WorthQueryProductBranch,
) -> Result<Option<RequiredWaveSelection<'runtime, Schema>>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let Some(anchor_ready) = runtime
        .runtime
        .output_demands
        .interest_ready_readmission(interest)?
    else {
        return Ok(None);
    };
    select_required_wave_from_ready(
        execution,
        runtime,
        anchor_ready,
        branch,
        RequiredWaveTarget::Caller,
    )
    .map(Some)
}

/// Only a live advance can select the basis following a completed publication.
fn reselect_required_wave<'runtime, Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    previous: LiveSelection<'_, Schema>,
) -> Result<RequiredWaveSelection<'runtime, Schema>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let RequiredWaveSelection {
        shared,
        positioned,
        anchor_ready,
        branch,
        target,
        state: _,
    } = previous.0;
    drop((shared, positioned));
    select_required_wave_from_ready(execution, runtime, anchor_ready, branch, target)
}

fn select_required_wave_from_ready<'runtime, Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    anchor_ready: SelectedReadyReadmission,
    branch: WorthQueryProductBranch,
    target: RequiredWaveTarget,
) -> Result<RequiredWaveSelection<'runtime, Schema>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let (shared, positioned) = select_required_basis(execution, runtime, branch)?;
    let head = runtime.runtime.output_demands.publication_head_selection(
        shared
            .selected()
            .product()
            .observation()
            .lifecycle_incarnation(),
    )?;
    Ok(RequiredWaveSelection::from_basis(
        shared,
        positioned,
        anchor_ready,
        branch,
        target,
        head,
    ))
}

/// The same owner-issued Product/native basis for both kinds of wave anchor.
pub fn select_required_basis<'runtime, Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &'runtime dyn crate::ports::ApplicationRuntimeRef<Schema>,
    branch: WorthQueryProductBranch,
) -> Result<
    (
        SharedSelectedProductOperation<'runtime, Schema>,
        PositionedRelationalSnapshot,
    ),
    WorthQueryOutputDemandDenial,
>
where
    Schema: ApplicationSchema + 'static,
{
    let runtime = runtime.runtime_inner();
    // A Ready the wave answers is read at this selection.
    let _held_world_observations = runtime
        .runtime
        .primary_provider
        .take_ready_read_snapshot_pressure()
        .then(|| runtime.hold_world_snapshot_pressure_for_test(branch));
    let selected = runtime
        .on_branch(branch)
        .select_with_request(execution)
        .map_err(|stop| {
            WorthQueryOutputDemandDenial::product_selection(
                stop,
                "required output Product basis could not be selected",
            )
        })?;
    let product = selected.product().read_lease_ref();
    let snapshot = selected.application_basis().snapshot_handle();
    let positioned = runtime
        .runtime
        .primary_provider
        .graph_ref()
        .with_runtime(execution, |native| {
            native.read_truth().positioned_snapshot(snapshot)
        })
        .map_err(position_denial)?;
    runtime
        .runtime
        .output_demands
        .observe_selected_discontinuity(
            product,
            &positioned,
            &runtime
                .runtime
                .primary_provider
                .graph_ref()
                .source_owner_ref()
                .invalidation_owner,
        )?;
    let shared = selected
        .prepare_shared_query_basis()
        .map_err(|(_, stop)| admission_denial(stop))?;
    Ok((shared, positioned))
}

/// A committed Ready moved the wave past its selected position.
pub fn committed_ready(
    ready: &SelectedReadyReadmission,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    Ok(matches!(
        &ready.completion().authority,
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Committed(_)
        | crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Recovered(_)
    ))
}
