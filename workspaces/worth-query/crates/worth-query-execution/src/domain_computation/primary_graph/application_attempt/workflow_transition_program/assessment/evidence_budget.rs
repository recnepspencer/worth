//! Streaming demand and retained-evidence admission before materialization.
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationRealizedEffect;
use crate::domain_computation::primary_graph::workflow::{
    schema::WorthQueryWorkflowLayout, workflow_evidence_retained_bytes,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

#[cfg(test)]
#[path = "evidence_budget/tests.rs"]
mod tests;

pub(super) fn admit(
    layout: &WorthQueryWorkflowLayout,
    allowance: u64,
    node_path: &str,
    visit: impl FnOnce(
        &mut dyn FnMut(
            WorthQueryApplicationRealizedEffect,
        ) -> Result<(), WorthQueryApplicationAttemptDenial>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>,
) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    let mut retained_bytes = 0_u64;
    visit(&mut |effect| {
        retained_bytes =
            retained_bytes.saturating_add(workflow_evidence_retained_bytes(layout, &effect));
        Ok(())
    })?;
    // The installed allowance includes its last byte. This same charge is
    // recorded durably and applied to the lineage's progress accounting.
    if retained_bytes > allowance {
        return Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceEvidenceCapacityUnavailable,
            node_path,
        ));
    }
    Ok(retained_bytes)
}
