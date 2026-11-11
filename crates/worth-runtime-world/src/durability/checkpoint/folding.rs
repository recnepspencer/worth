//! Fold the admitted selected image and only its original uncovered W tails.
use super::{WorldCheckpointCoverage, WorldCheckpointImage, WorldCheckpointReadSource};
use crate::durability::{WorldDurableHomeBinding, WorldReadmissionDenial as Denial};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::WorldDurableJournal;

impl WorldCheckpointImage {
    pub(crate) fn read_protected_source(
        source: &dyn WorldCheckpointReadSource,
        binding: &WorldDurableHomeBinding,
        request: &ExecutionRequest<'_, '_>,
        selected_native: Option<
            &mut Option<
                std::sync::Arc<worth_relational::facade::durability::RelationalWorldSourceCatalog>,
            >,
        >,
    ) -> Result<(Self, WorldCheckpointCoverage), Denial> {
        let mut view = source.read(request).map_err(Denial::Medium)?;
        let has_selected = view.selected.is_some();
        let mut image = match view.selected.take() {
            Some((bytes, region)) => {
                let bytes = bytes.get(region).ok_or(Denial::Medium(
                    crate::durability::WorldDurableFrameDenial::Malformed.into(),
                ))?;
                Self::decode_selected(bytes, binding, request)?
            }
            None => Self::empty(binding),
        };
        let selected_coverage = image.coverage.clone();
        if let Some(output) = selected_native {
            *output = if has_selected {
                Some(
                    crate::durability::prepared_home_source::native_catalog::prepare(
                        &image, request, None,
                    )?,
                )
            } else {
                None
            };
        }
        for journal in [
            WorldDurableJournal::Branches,
            WorldDurableJournal::Custody,
            WorldDurableJournal::Occurrences,
        ] {
            let covered = image.coverage.journal(journal).position;
            let head = view.head(journal);
            if covered > head {
                return Err(Denial::JournalSourceMismatch);
            }
            let mut position = covered;
            while position < head {
                position = position
                    .checked_add(1)
                    .ok_or(Denial::JournalSourceMismatch)?;
                let record = source
                    .read_journal_record(request, journal, position)
                    .map_err(Denial::Medium)?;
                if record.position != position {
                    return Err(Denial::JournalSourceMismatch);
                }
                image.admit_journal_record(
                    journal,
                    position,
                    record.digest,
                    record.bytes.as_ref(),
                    request,
                )?;
            }
        }
        image.resolve_collected_history(request)?;
        image.validate_open_identities(request)?;
        Ok((image, selected_coverage))
    }
}
