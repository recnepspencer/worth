//! Rebinds the one program-activation cell to recovered authoritative truth.

use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_query_installation::facade::WorthQueryProgramSupportRoster;

use super::super::program_occurrence::{
    program_revision_rendering, WorthQueryProgramActivationCell,
};
use super::super::{
    WorthQueryPrimaryGraph, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

/// The phase witnesses the caller's request for the inline recovery read.
pub(in crate::domain_computation::primary_graph) fn recover_program_activation<Schema>(
    _phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &WorthQueryPrimaryGraph,
    roster: &WorthQueryProgramSupportRoster<Schema>,
    cell: &WorthQueryProgramActivationCell,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let layout = graph.layout.program_activation().clone();
    let identity = graph.integration_handle().with_runtime(|runtime| {
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .map_err(|error| denial(format!("recovered branch basis refused: {error:?}")))?;
        let branch = runtime
            .read_truth()
            .project_observation(&basis.observation())
            .map_err(|error| denial(format!("recovered branch is unreadable: {error:?}")))?;
        read_activation(
            &branch,
            &layout,
            usize::MAX,
            "recovered program activation is not in the admitted roster",
            |rendering| {
                roster
                    .entries()
                    .iter()
                    .any(|entry| program_revision_rendering(entry.revision()) == *rendering)
            },
        )
        .map(|(identity, _)| identity)
    })?;
    cell.publish(identity)
        .map_err(|_| denial("recovered program activation was already published"))
}

/// Reads the activation on the caller's exact branch view, with native scan accounting.
pub(super) fn read_activation(
    branch: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    layout: &super::super::schema_layout::WorthQueryProgramActivationLayout,
    maximum_work_units: usize,
    mismatch_subject: &str,
    accepts: impl FnOnce(&worth_foundational::facade::AspectValue) -> bool,
) -> Result<
    (worth_relational::facade::identity::EntityId, usize),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let read = branch
        .bounded_entities_of_kind(layout.entity_kind, maximum_work_units)
        .map_err(|_| denial("recovered program activation scan exceeded its bound"))?;
    let work = read.work_units();
    let records = read.into_records();
    let [record] = records.as_slice() else {
        return Err(denial(format!(
            "recovered branch contains {} program activation records, expected one",
            records.len()
        )));
    };
    let state = record
        .authoritative_aspect_state
        .as_ref()
        .ok_or_else(|| denial("recovered program activation has no authoritative state"))?;
    let value = state
        .get(layout.program_revision_locator.aspect().aspect_key())
        .ok_or_else(|| denial("recovered program activation has no revision aspect"))?;
    let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
        return Err(denial(
            "recovered program activation has foreign aspect shape",
        ));
    };
    let field = layout
        .program_revision_locator
        .field_path()
        .fields()
        .first()
        .ok_or_else(|| denial("program activation layout has no revision field"))?;
    let rendering = fields
        .get(field)
        .ok_or_else(|| denial("recovered program activation has no revision"))?;
    if !accepts(rendering) {
        return Err(denial(mismatch_subject));
    }
    Ok((record.entity_id, work))
}

fn denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        subject,
    )
}
