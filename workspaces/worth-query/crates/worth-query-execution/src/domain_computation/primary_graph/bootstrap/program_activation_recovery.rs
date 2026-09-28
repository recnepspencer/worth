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

pub(in crate::domain_computation::primary_graph) fn recover_program_activation<Schema>(
    graph: &WorthQueryPrimaryGraph,
    roster: &WorthQueryProgramSupportRoster<Schema>,
    cell: &WorthQueryProgramActivationCell,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let layout = graph.layout.program_activation().clone();
    let identity = graph.integration_handle().with_runtime(|runtime| {
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .map_err(|error| denial(format!("recovered branch basis refused: {error:?}")))?;
        // Read on main's own root: a version alone would also see a
        // sibling's later adoption of the same activation record. Recovery
        // examines the recovered root once, as the unbounded scan it replaces.
        let records = runtime
            .read_truth()
            .project_observation(&basis.observation())
            .map_err(|error| denial(format!("recovered branch is unreadable: {error:?}")))?
            .bounded_entities_of_kind(layout.entity_kind, usize::MAX)
            .map_err(|_| denial("recovered program activation scan exceeded its bound"))?
            .into_records();
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
        if !roster
            .entries()
            .iter()
            .any(|entry| program_revision_rendering(entry.revision()) == *rendering)
        {
            return Err(denial(
                "recovered program activation is not in the admitted roster",
            ));
        }
        Ok(record.entity_id)
    })?;
    cell.publish(identity)
        .map_err(|_| denial("recovered program activation was already published"))
}

fn denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        subject,
    )
}
