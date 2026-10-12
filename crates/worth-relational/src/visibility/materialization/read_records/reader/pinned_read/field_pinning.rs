//! Selected field admission captures registry handles before any row work.
use super::super::VisibilityReadContext;
use super::*;
use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind as Kind, SelectedIndexGenerationAdmissionStop,
    SelectedIndexReadWork,
};
use crate::runtime::{ExactLookupInputs, VisibilityProjectionView};
impl VisibilityReadContext<'_> {
    pub(crate) fn pin_selected_field_read_capability<E>(
        &self,
        view: &VisibilityProjectionView,
        index: DerivedIndexId,
        locator: &worth_foundational::facade::AspectFieldLocator,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut prepare: impl FnMut(SelectedIndexReadWork, u64) -> Result<(), E>,
    ) -> Result<RelationalPinnedRead<()>, Stop<E>> {
        let denied = |kind| Stop::Lookup(BoundedEntityFieldLookupDenial::new(kind, index));
        let mut pinned = self
            .pin_view_read_capability(view, &[], execution)
            .map_err(|stop| match stop {
                RelationalPinnedReadAdmissionDenial::SnapshotUnavailable => {
                    Stop::ExactBasisRequired
                }
                RelationalPinnedReadAdmissionDenial::IndexNotInstalled(_) => {
                    denied(Kind::IndexNotInstalled)
                }
                RelationalPinnedReadAdmissionDenial::Request(stop) => {
                    denied(Kind::Execution(crate::execution::denial::kernel_stop(stop)))
                }
            })?;
        let selected = self
            .runtime
            .indexes
            .exact_lookup_inputs_admitted(
                index,
                locator,
                pinned.basis.branch_id(),
                pinned.basis.version_id(),
                pinned.context.schema_version,
                &mut prepare,
            )
            .map_err(|stop| match stop {
                SelectedIndexGenerationAdmissionStop::Admission(stop) => Stop::Admission(stop),
                SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                    Stop::AccountingOverflow
                }
            })?;
        let (definition, generation) = match selected {
            ExactLookupInputs::MissingDefinition => return Err(denied(Kind::IndexNotInstalled)),
            ExactLookupInputs::MissingGeneration => {
                return Err(denied(Kind::ExactGenerationUnavailable))
            }
            ExactLookupInputs::WrongKind => return Err(denied(Kind::WrongIndexKind)),
            ExactLookupInputs::Ready {
                definition,
                generation,
            } => (definition, generation),
        };
        pinned.indexes.insert(
            index,
            PinnedIndex {
                definition,
                generation: Some(generation),
            },
        );
        Ok(pinned)
    }
}
