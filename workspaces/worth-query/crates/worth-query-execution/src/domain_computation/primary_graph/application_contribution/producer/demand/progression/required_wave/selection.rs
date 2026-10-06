use super::*;

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

/// A performed checkpoint moves the Native head. The caller retains the
/// authentic original Ready pin while the next frame selects a fresh Product.
pub(super) fn reselect_required_wave<'runtime, Schema>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    previous: RequiredWaveSelection<'_, Schema>,
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
    } = previous;
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
    Ok(RequiredWaveSelection {
        shared,
        positioned,
        anchor_ready,
        branch,
        target,
    })
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
