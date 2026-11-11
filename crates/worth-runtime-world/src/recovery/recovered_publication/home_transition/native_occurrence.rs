//! Admit the actual positioned occurrence after its real acknowledgement.
use super::*;
use crate::durability::{AdmittedWorldOccurrence, WorldCheckpointImage, WorldRecord};
use crate::publication::WorldHomeTransitionEffectDenial as Denial;
use worth_execution::ExecutionRequest;

impl SubmittedContinuationWork {
    pub(super) fn admit_occurrence(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<AdmittedWorldOccurrence, Denial> {
        let common = &self.work.common;
        let source = common
            .binding
            .capture_checkpoint_source(request)
            .map_err(Denial::Medium)?;
        let (image, _) = WorldCheckpointImage::read_protected_source(
            source.as_ref(),
            common.binding.binding(),
            request,
            None,
        )
        .map_err(Denial::Readmission)?;
        let WorldRecord::Occurrence { open, owners, .. } = common
            .occurrence
            .subject
            .as_ref()
            .expect("the original occurrence is finalized before submission")
            .record()
        else {
            unreachable!("only an original occurrence enters this controller")
        };
        let mut matches = image
            .occurrences
            .values()
            .filter(|row| row.open().association == open.association);
        let occurrence = matches.next().ok_or(Denial::NativeEffectMismatch)?;
        let relational = owners.relational().ok_or(Denial::NativeEffectMismatch)?;
        if matches.next().is_some()
            || occurrence.open() != open
            || occurrence.relational() != Some(relational)
        {
            return Err(Denial::NativeEffectMismatch);
        }
        let actual = self.work.native.custody().effect().subject();
        if relational.commit != actual.receipt().commit_id.0
            || relational.position != actual.position().0
            || &relational.digest != actual.artifact_digest()
        {
            return Err(Denial::NativeEffectMismatch);
        }
        Ok(occurrence.clone())
    }

    pub(super) fn materialize(
        self,
        occurrence: AdmittedWorldOccurrence,
    ) -> MaterializedContinuationWork {
        let NativeContinuationWork {
            common,
            native,
            record_backing,
        } = self.work;
        let record = crate::history::ReadmittedPublicationRecord::new_native(
            record_backing,
            common.before.snapshot().clone(),
            common.after.clone(),
            common.attempt.clone(),
            occurrence.location().clone(),
            occurrence.publishing_incarnation(),
            native,
        );
        MaterializedContinuationWork {
            common,
            record,
            permit: self.permit,
        }
    }
}
