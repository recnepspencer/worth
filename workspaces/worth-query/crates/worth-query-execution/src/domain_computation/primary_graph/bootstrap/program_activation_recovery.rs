//! Reads the one program-activation record recovered authoritative truth holds.

use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};
use worth_relational::facade::identity::EntityId;

use super::super::{
    WorthQueryPrimaryGraph, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

/// The activation a recovered image recorded: its row and the revision it names.
pub(in crate::domain_computation::primary_graph) struct RecordedProgramActivation {
    pub(in crate::domain_computation::primary_graph) identity: EntityId,
    pub(in crate::domain_computation::primary_graph) rendering: AspectValue,
}

/// Reads the recovered main branch's activation. A program install seeds it
/// before any bootstrap row and a declaration install never does, so its
/// presence is what makes an image a program image.
pub(in crate::domain_computation::primary_graph) fn recorded_program_activation(
    graph: &WorthQueryPrimaryGraph,
) -> Result<Option<RecordedProgramActivation>, WorthQueryPrimaryGraphInstallationDenial> {
    let layout = graph.layout.program_activation().clone();
    graph.integration_handle().with_runtime(|runtime| {
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .map_err(|error| denial(format!("recovered branch basis refused: {error:?}")))?;
        let branch = runtime
            .read_truth()
            .project_observation(&basis.observation())
            .map_err(|error| denial(format!("recovered branch is unreadable: {error:?}")))?;
        scan_activation(&branch, &layout, usize::MAX).map(|(recorded, _)| recorded)
    })
}

/// Reads the activation on the caller's exact branch view, with native scan accounting.
pub(super) fn read_activation(
    branch: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    layout: &super::super::schema_layout::WorthQueryProgramActivationLayout,
    maximum_work_units: usize,
    mismatch_subject: &str,
    accepts: impl FnOnce(&AspectValue) -> bool,
) -> Result<(EntityId, usize), WorthQueryPrimaryGraphInstallationDenial> {
    let (recorded, work) = scan_activation(branch, layout, maximum_work_units)?;
    let recorded =
        recorded.ok_or_else(|| denial("recovered branch contains no program activation record"))?;
    if !accepts(&recorded.rendering) {
        return Err(denial(mismatch_subject));
    }
    Ok((recorded.identity, work))
}

fn scan_activation(
    branch: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    layout: &super::super::schema_layout::WorthQueryProgramActivationLayout,
    maximum_work_units: usize,
) -> Result<(Option<RecordedProgramActivation>, usize), WorthQueryPrimaryGraphInstallationDenial> {
    let read = branch
        .bounded_entities_of_kind(layout.entity_kind, maximum_work_units)
        .map_err(|_| denial("recovered program activation scan exceeded its bound"))?;
    let work = read.work_units();
    let records = read.into_records();
    let record = match records.as_slice() {
        [] => return Ok((None, work)),
        [record] => record,
        records => {
            return Err(denial(format!(
                "recovered branch contains {} program activation records, expected at most one",
                records.len()
            )))
        }
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
    Ok((
        Some(RecordedProgramActivation {
            identity: record.entity_id,
            rendering: rendering.clone(),
        }),
        work,
    ))
}

fn denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        subject,
    )
}
