use worth_ui_host_contract::UiHostObservationBatch;

use super::basis_admission::UiBasisAdmittedObservationBatch;
use super::sequence_coverage::UiSequenceCoveredObservationBatch;
use super::state::UiHostObservationBatchFingerprint;
use super::structural_admission::UiStructurallyAdmittedObservationBatch;
use super::{
    UiHostObservationReportDenial, UiHostObservationReportOutcome,
    UiHostObservationReportValidation, UiQuarantinedHostObservationBatch,
};

pub(crate) struct UiHostObservationValidationContext<'a> {
    pub(crate) host_session: u64,
    pub(crate) protocol: worth_ui_host_contract::UiHostProtocolAgreement,
    pub(crate) mounted: crate::mounting::UiMountedObservationValidationBasis<'a>,
}

impl UiHostObservationReportValidation {
    pub(crate) fn validate(
        &mut self,
        batch: UiHostObservationBatch,
        context: UiHostObservationValidationContext<'_>,
    ) -> UiHostObservationReportOutcome {
        let declared_work = super::work_report::UiHostObservationDeclaredWork::from_batch(&batch);
        let outcome = match self.try_validate(batch, context) {
            Ok(outcome) => outcome,
            Err(denial) => UiHostObservationReportOutcome::Denied(denial),
        };
        self.work.record(declared_work, &outcome);
        outcome
    }

    fn try_validate(
        &mut self,
        batch: UiHostObservationBatch,
        context: UiHostObservationValidationContext<'_>,
    ) -> Result<UiHostObservationReportOutcome, UiHostObservationReportDenial> {
        if self.shutdown {
            return Err(UiHostObservationReportDenial::Shutdown);
        }
        let admitted = UiStructurallyAdmittedObservationBatch::admit(batch, context.protocol)?;
        let core = admitted.core();
        let covered = UiSequenceCoveredObservationBatch::prove(admitted)?;
        // A denial loses this batch's reports and leaves the cursor where it
        // was. The host never resends it, so a denial of this session's next
        // batch lets the host continue after it rather than strand every later
        // batch behind a gap. A foreign or unproven batch records nothing.
        let next_in_sequence = self.validate_sequence_progression(core.sequences()).is_ok();
        let own_session = core.host_session() == context.host_session;
        let outcome = self.admit_covered_batch(core, covered, context);
        if outcome.is_err() && next_in_sequence && own_session {
            self.pass_denied_sequence(core.sequences().last());
        }
        outcome
    }

    fn admit_covered_batch(
        &mut self,
        core: worth_ui_host_contract::UiHostObservationCanonicalCore,
        covered: UiSequenceCoveredObservationBatch,
        context: UiHostObservationValidationContext<'_>,
    ) -> Result<UiHostObservationReportOutcome, UiHostObservationReportDenial> {
        let integrity = covered.integrity();
        if self.is_rejected(core.frame()) {
            return Err(UiHostObservationReportDenial::RejectedFrame);
        }
        if self.is_never_presented(core.frame()) {
            return Err(UiHostObservationReportDenial::NeverPresentedFrame);
        }
        if let Some(duplicate) = self.duplicate_covered_batch(&covered) {
            return Ok(duplicate);
        }
        if self.is_indeterminate(core.frame(), core.binding()) {
            return self.quarantine(core, integrity);
        }
        self.validate_sequence_progression(core.sequences())?;
        let basis = UiBasisAdmittedObservationBatch::admit(
            covered,
            context.mounted.retention(),
            context.host_session,
            self.observation_basis(core.frame()),
        )?;
        if context
            .mounted
            .binding_requires_reconciliation(core.binding())
        {
            return self.quarantine(core, integrity);
        }
        self.retain_covered_batch(basis)
    }

    fn quarantine(
        &mut self,
        core: worth_ui_host_contract::UiHostObservationCanonicalCore,
        integrity: worth_ui_host_contract::UiHostObservationIntegrity,
    ) -> Result<UiHostObservationReportOutcome, UiHostObservationReportDenial> {
        if self.quarantine.len() >= self.capacity.quarantined_batches() {
            return Err(UiHostObservationReportDenial::QuarantineCountCapacityExceeded);
        }
        let required_bytes = self
            .quarantine_bytes
            .checked_add(super::state::quarantine_entry_structural_bytes())
            .ok_or(UiHostObservationReportDenial::QuarantineAccountingOverflow)?;
        if required_bytes > self.capacity.quarantined_bytes() {
            return Err(UiHostObservationReportDenial::QuarantineByteCapacityExceeded);
        }
        let quarantined = UiQuarantinedHostObservationBatch::new(core);
        self.quarantine.push_back(quarantined);
        self.quarantine_fingerprints
            .push_back(UiHostObservationBatchFingerprint {
                sequences: core.sequences(),
                integrity,
            });
        self.quarantine_bytes = required_bytes;
        self.advance_sequence(core.sequences().last());
        Ok(UiHostObservationReportOutcome::Quarantined(quarantined))
    }
}
