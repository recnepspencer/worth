use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryGraphCompositionAdmissionTraceStage {
    ProgramParsed,
    SymbolsValidated,
    LoweringValidated,
    CapabilityFamilyClassified,
    SupportPostureResolved,
    IdentityPreservationEvaluated,
    VerificationSubstrateEvaluated,
    DomainInvariantEvaluated,
    DeniedBeforeExecution,
}

impl WorthQueryGraphCompositionAdmissionTraceStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProgramParsed => "program_parsed",
            Self::SymbolsValidated => "symbols_validated",
            Self::LoweringValidated => "lowering_validated",
            Self::CapabilityFamilyClassified => "capability_family_classified",
            Self::SupportPostureResolved => "support_posture_resolved",
            Self::IdentityPreservationEvaluated => "identity_preservation_evaluated",
            Self::VerificationSubstrateEvaluated => "verification_substrate_evaluated",
            Self::DomainInvariantEvaluated => "domain_invariant_evaluated",
            Self::DeniedBeforeExecution => "denied_before_execution",
        }
    }
}

impl std::fmt::Display for WorthQueryGraphCompositionAdmissionTraceStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionAdmissionTrace {
    stages: Vec<WorthQueryGraphCompositionAdmissionTraceStage>,
    failure_stage: WorthQueryGraphCompositionAdmissionTraceStage,
    admission_trace_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryGraphCompositionAdmissionTrace {
    pub(in crate::runtime) fn new(
        stages: Vec<WorthQueryGraphCompositionAdmissionTraceStage>,
        failure_stage: WorthQueryGraphCompositionAdmissionTraceStage,
    ) -> Result<Self, IdentityDerivationDenial> {
        assert!(
            !stages.is_empty(),
            "graph composition admission trace must include at least one stage"
        );
        assert!(
            stages.contains(&failure_stage),
            "graph composition admission trace failure stage must appear in the stage list"
        );
        assert_eq!(
            stages.last(),
            Some(&WorthQueryGraphCompositionAdmissionTraceStage::DeniedBeforeExecution),
            "graph composition admission trace must end at denied-before-execution"
        );
        assert_ne!(
            failure_stage,
            WorthQueryGraphCompositionAdmissionTraceStage::DeniedBeforeExecution,
            "graph composition admission trace failure stage must name the stage that failed, not the terminal denial marker"
        );
        let admission_trace_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-admission-trace",
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("stage"),
                    stages.iter().map(|stage| stage.as_str()),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("failure_stage"),
                    failure_stage.as_str(),
                )
                .seal()?;
        Ok(Self {
            stages,
            failure_stage,
            admission_trace_digest,
        })
    }

    pub fn stages(&self) -> &[WorthQueryGraphCompositionAdmissionTraceStage] {
        &self.stages
    }

    pub fn failure_stage(&self) -> WorthQueryGraphCompositionAdmissionTraceStage {
        self.failure_stage
    }

    pub fn admission_trace_digest(&self) -> &str {
        self.admission_trace_digest.as_str()
    }

    pub fn admission_trace_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_trace_digest
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WorthQueryGraphCompositionAdmissionTrace, WorthQueryGraphCompositionAdmissionTraceStage,
    };

    #[test]
    #[should_panic(expected = "failure stage must appear in the stage list")]
    fn admission_trace_rejects_failure_stage_outside_stage_list(
    ) -> Result<(), IdentityDerivationDenial> {
        let _ = WorthQueryGraphCompositionAdmissionTrace::new(
            vec![
                WorthQueryGraphCompositionAdmissionTraceStage::ProgramParsed,
                WorthQueryGraphCompositionAdmissionTraceStage::DeniedBeforeExecution,
            ],
            WorthQueryGraphCompositionAdmissionTraceStage::SymbolsValidated,
        )?;

        Ok(())
    }

    #[test]
    #[should_panic(expected = "must end at denied-before-execution")]
    fn admission_trace_rejects_non_terminal_denied_marker() -> Result<(), IdentityDerivationDenial>
    {
        let _ = WorthQueryGraphCompositionAdmissionTrace::new(
            vec![WorthQueryGraphCompositionAdmissionTraceStage::ProgramParsed],
            WorthQueryGraphCompositionAdmissionTraceStage::ProgramParsed,
        )?;

        Ok(())
    }
}
