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
    let Some(caller_ready) = runtime
        .output_demands
        .interest_ready_readmission(interest, admission)?
    else {
        return Ok(None);
    };
    select_required_wave_from_ready(runtime, caller_ready, branch, admission).map(Some)
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
        caller_ready,
        branch,
    } = previous;
    drop((shared, positioned));
    select_required_wave_from_ready(runtime, caller_ready, branch, admission)
}

fn select_required_wave_from_ready<'runtime, Schema>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    caller_ready: SelectedReadyReadmission,
    branch: WorthQueryProductBranch,
    admission: &mut InvalidationEditAdmission,
) -> Result<RequiredWaveSelection<'runtime, Schema>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
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
    Ok(RequiredWaveSelection {
        shared,
        positioned,
        caller_ready,
        branch,
    })
}
