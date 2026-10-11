use super::*;

/// A caller's Ready or an exactly requested upstream Ready anchors the wave.
/// `shared` and `positioned` retain its admitted Product/native proof basis.
pub(super) struct RequiredWaveSelection<'runtime, Schema> {
    pub(super) shared: SharedSelectedProductOperation<'runtime, Schema>,
    pub(super) positioned: PositionedRelationalSnapshot,
    pub(super) anchor_ready: SelectedReadyReadmission,
    pub(super) branch: WorthQueryProductBranch,
    pub(super) target: RequiredWaveTarget,
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
    pub(super) fn freeze(&mut self) {
        self.state = SelectionState::Frozen;
    }
    pub(super) fn is_frozen(&self) -> bool {
        matches!(self.state, SelectionState::Frozen)
    }
    pub(super) fn reselect(
        self,
        runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        match self.state {
            SelectionState::Frozen => Ok(self),
            SelectionState::Live(_) => {
                reselect_required_wave(runtime, LiveSelection(self), admission)
            }
        }
    }
    /// Only actual custody delivery can thaw; unrelated entries stay frozen.
    pub(super) fn after_awaited_delivery(
        self,
        runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        _delivery: super::super::super::required_continuations::DeliveredAwaited,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let head = runtime.output_demands.publication_head_selection(
            self.shared
                .selected()
                .product()
                .observation()
                .lifecycle_incarnation(),
            admission,
        )?;
        match head {
            crate::domain_computation::primary_graph::application_output_demand::PublicationHeadSelection::Frozen => Ok(self),
            crate::domain_computation::primary_graph::application_output_demand::PublicationHeadSelection::Live(_) =>
                select_required_wave_from_ready(runtime, self.anchor_ready, self.branch, self.target, admission),
        }
    }
    pub(super) fn from_basis(
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

pub(super) fn select_required_wave<'runtime, Schema>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    interest: &WorthQueryOutputDemandInterest,
    branch: WorthQueryProductBranch,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<RequiredWaveSelection<'runtime, Schema>>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let Some(anchor_ready) = runtime
        .output_demands
        .interest_ready_readmission(interest, admission)?
    else {
        return Ok(None);
    };
    select_required_wave_from_ready(
        runtime,
        anchor_ready,
        branch,
        RequiredWaveTarget::Caller,
        admission,
    )
    .map(Some)
}

/// Only a live advance can select the basis following a completed publication.
fn reselect_required_wave<'runtime, Schema>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    previous: LiveSelection<'_, Schema>,
    admission: &mut InvalidationEditAdmission,
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
    select_required_wave_from_ready(runtime, anchor_ready, branch, target, admission)
}

fn select_required_wave_from_ready<'runtime, Schema>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    anchor_ready: SelectedReadyReadmission,
    branch: WorthQueryProductBranch,
    target: RequiredWaveTarget,
    admission: &mut InvalidationEditAdmission,
) -> Result<RequiredWaveSelection<'runtime, Schema>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let (shared, positioned) = select_required_basis(runtime, branch, admission)?;
    let head = runtime.output_demands.publication_head_selection(
        shared
            .selected()
            .product()
            .observation()
            .lifecycle_incarnation(),
        admission,
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
pub(super) fn select_required_basis<'runtime, Schema>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    admission: &mut InvalidationEditAdmission,
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
    // A Ready the wave answers is read at this selection.
    #[cfg(feature = "test-primary-graph-faults")]
    let _held_world_observations = runtime
        .primary_provider
        .take_ready_read_snapshot_pressure()
        .then(|| runtime.hold_world_snapshot_pressure_for_test(branch));
    let selected = runtime.on_branch(branch).select().map_err(|stop| {
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ProductSelection(stop),
            "required output Product basis could not be selected",
        )
    })?;
    let product = selected.product().read_lease_ref();
    let snapshot = selected.application_basis().snapshot_handle();
    let positioned = runtime
        .primary_provider
        .graph
        .with_runtime(|native| {
            native
                .read_truth()
                .positioned_snapshot_admitted(snapshot, |work, bytes| {
                    admission.charge_external_work(work)?;
                    admission.admit_read_scratch(bytes)
                })
        })
        .map_err(position_denial)?;
    runtime.output_demands.observe_selected_discontinuity(
        product,
        &positioned,
        &runtime
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner,
        admission,
    )?;
    let shared = selected
        .prepare_shared_query_basis(admission)
        .map_err(|(_, stop)| admission_denial(stop))?;
    Ok((shared, positioned))
}

/// A committed Ready moved the wave past its selected position.
pub(super) fn committed_ready(
    ready: &SelectedReadyReadmission,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(2)
        .map_err(|_| work_denial())?;
    Ok(matches!(
        &ready.completion().authority,
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Committed(_)
    ))
}
