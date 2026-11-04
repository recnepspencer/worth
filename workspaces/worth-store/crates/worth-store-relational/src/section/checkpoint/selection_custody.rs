//! Actual source/member selection custody, independent of serialized section DATA.
use super::*;
use std::sync::{Arc, Mutex};

pub(super) struct SectionSelectionCustody {
    pub(super) source: Arc<crate::cold_source::StoreColdSource>,
    pub(super) retained: crate::cold_source::RetainedMemberPublication,
    pub(super) selection: Option<Arc<dyn super::super::RelationalCheckpointSelectionSource>>,
    pub(super) replacement: Mutex<crate::cold_source::selected_replacement::SelectedReplacement>,
}

impl SectionSelectionCustody {
    pub(super) fn prepare(
        source: &Arc<crate::cold_source::StoreColdSource>,
        checkpoint: &RelationalNativeCheckpoint,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalSectionCheckpointDenial> {
        let retained = source
            .protect_member_selection_for_request(
                checkpoint.members().iter().map(|member| member.locator()),
                execution,
            )
            .map_err(RelationalSectionCheckpointDenial::ReadProtection)?;
        Ok(Self {
            source: source.clone(),
            retained,
            selection: None,
            replacement: Mutex::default(),
        })
    }
}
