use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryGraphCompositionAdmissionTrace, WorthQueryGraphCompositionAdmissionTraceStage,
    WorthQueryMutationTargetCollectionIdentity, WorthQueryRuntimeError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryGraphCompositionDenialKind {
    EmptyComposition,
    DuplicateSymbolDeclaration,
    UnresolvedSymbolicReference,
    SymbolicCollectionMismatch,
    ExistingTargetBindingUnsupported,
    ExistingTargetResolvedTargetMissing,
    ExistingTargetCollectionMismatch,
    ExistingTargetRetargetUnsupported,
    ExistingTargetIdentityPreservationUnavailable,
    ExistingTargetSupersessionUnsupported,
    ExistingTargetBackendVerificationUnsupported,
    ExistingTargetClearAssertionUnsupported,
    ExistingTargetMissingAssertedAspect,
    ExistingTargetAssertedValueMismatch,
}

impl WorthQueryGraphCompositionDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmptyComposition => "empty-composition",
            Self::DuplicateSymbolDeclaration => "duplicate-symbol-declaration",
            Self::UnresolvedSymbolicReference => "unresolved-symbolic-reference",
            Self::SymbolicCollectionMismatch => "symbolic-collection-mismatch",
            Self::ExistingTargetBindingUnsupported => "existing-target-binding-unsupported",
            Self::ExistingTargetResolvedTargetMissing => "existing-target-resolved-target-missing",
            Self::ExistingTargetCollectionMismatch => "existing-target-collection-mismatch",
            Self::ExistingTargetRetargetUnsupported => "existing-target-retarget-unsupported",
            Self::ExistingTargetIdentityPreservationUnavailable => {
                "existing-target-identity-preservation-unavailable"
            }
            Self::ExistingTargetSupersessionUnsupported => {
                "existing-target-supersession-unsupported"
            }
            Self::ExistingTargetBackendVerificationUnsupported => {
                "existing-target-backend-verification-unsupported"
            }
            Self::ExistingTargetClearAssertionUnsupported => {
                "existing-target-clear-assertion-unsupported"
            }
            Self::ExistingTargetMissingAssertedAspect => "existing-target-missing-asserted-aspect",
            Self::ExistingTargetAssertedValueMismatch => "existing-target-asserted-value-mismatch",
        }
    }
}

impl std::fmt::Display for WorthQueryGraphCompositionDenialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionDenial {
    kind: WorthQueryGraphCompositionDenialKind,
    symbol: Option<String>,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    admission_trace: WorthQueryGraphCompositionAdmissionTrace,
    message: String,
    denial_digest: String,
}

impl WorthQueryGraphCompositionDenial {
    pub(crate) fn new(
        kind: WorthQueryGraphCompositionDenialKind,
        symbol: Option<String>,
        target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
        message: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let message = message.into();
        let admission_trace = default_admission_trace(kind)?;
        let denial_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-denial",
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .optional_value(WorthQueryEvidenceTag::new("symbol"), symbol.as_deref())
                .optional_value(
                    WorthQueryEvidenceTag::new("target_collection"),
                    target_collection
                        .as_ref()
                        .map(WorthQueryMutationTargetCollectionIdentity::as_str),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("trace"),
                    admission_trace.admission_trace_digest(),
                )
                .field_value(WorthQueryEvidenceTag::new("message"), &message)
                .seal()?
                .as_str()
                .to_string();
        Ok(Self {
            kind,
            symbol,
            target_collection,
            admission_trace,
            message,
            denial_digest,
        })
    }

    pub fn kind(&self) -> WorthQueryGraphCompositionDenialKind {
        self.kind
    }

    pub fn symbol(&self) -> Option<&str> {
        self.symbol.as_deref()
    }

    pub fn target_collection(&self) -> Option<&str> {
        self.target_collection
            .as_ref()
            .map(WorthQueryMutationTargetCollectionIdentity::as_str)
    }

    pub fn target_collection_identity(
        &self,
    ) -> Option<&WorthQueryMutationTargetCollectionIdentity> {
        self.target_collection.as_ref()
    }

    pub fn admission_trace(&self) -> &WorthQueryGraphCompositionAdmissionTrace {
        &self.admission_trace
    }

    pub fn failure_stage(&self) -> WorthQueryGraphCompositionAdmissionTraceStage {
        self.admission_trace.failure_stage()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn denial_digest(&self) -> &str {
        &self.denial_digest
    }
}

impl std::fmt::Display for WorthQueryGraphCompositionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.symbol() {
            Some(symbol) => write!(
                f,
                "graph composition denied for {} on symbol `{}`: {}",
                self.kind, symbol, self.message
            ),
            None => write!(
                f,
                "graph composition denied for {}: {}",
                self.kind, self.message
            ),
        }
    }
}

pub(crate) fn graph_composition_error(
    kind: WorthQueryGraphCompositionDenialKind,
    symbol: Option<String>,
    target_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    message: impl Into<String>,
) -> Result<WorthQueryRuntimeError, IdentityDerivationDenial> {
    Ok(WorthQueryRuntimeError::GraphCompositionDenied(
        WorthQueryGraphCompositionDenial::new(kind, symbol, target_collection, message)?,
    ))
}

fn default_admission_trace(
    kind: WorthQueryGraphCompositionDenialKind,
) -> Result<WorthQueryGraphCompositionAdmissionTrace, IdentityDerivationDenial> {
    use WorthQueryGraphCompositionAdmissionTraceStage as Stage;

    match kind {
        WorthQueryGraphCompositionDenialKind::EmptyComposition => {
            WorthQueryGraphCompositionAdmissionTrace::new(
                vec![Stage::ProgramParsed, Stage::DeniedBeforeExecution],
                Stage::ProgramParsed,
            )?
        }
        WorthQueryGraphCompositionDenialKind::DuplicateSymbolDeclaration => {
            WorthQueryGraphCompositionAdmissionTrace::new(
                vec![
                    Stage::ProgramParsed,
                    Stage::SymbolsValidated,
                    Stage::DeniedBeforeExecution,
                ],
                Stage::SymbolsValidated,
            )?
        }
        WorthQueryGraphCompositionDenialKind::UnresolvedSymbolicReference
        | WorthQueryGraphCompositionDenialKind::SymbolicCollectionMismatch
        | WorthQueryGraphCompositionDenialKind::ExistingTargetBindingUnsupported
        | WorthQueryGraphCompositionDenialKind::ExistingTargetResolvedTargetMissing
        | WorthQueryGraphCompositionDenialKind::ExistingTargetCollectionMismatch
        | WorthQueryGraphCompositionDenialKind::ExistingTargetRetargetUnsupported
        | WorthQueryGraphCompositionDenialKind::ExistingTargetIdentityPreservationUnavailable
        | WorthQueryGraphCompositionDenialKind::ExistingTargetSupersessionUnsupported => {
            WorthQueryGraphCompositionAdmissionTrace::new(
                vec![
                    Stage::ProgramParsed,
                    Stage::SymbolsValidated,
                    Stage::LoweringValidated,
                    Stage::DeniedBeforeExecution,
                ],
                Stage::LoweringValidated,
            )?
        }
        WorthQueryGraphCompositionDenialKind::ExistingTargetBackendVerificationUnsupported
        | WorthQueryGraphCompositionDenialKind::ExistingTargetClearAssertionUnsupported
        | WorthQueryGraphCompositionDenialKind::ExistingTargetMissingAssertedAspect
        | WorthQueryGraphCompositionDenialKind::ExistingTargetAssertedValueMismatch => {
            WorthQueryGraphCompositionAdmissionTrace::new(
                vec![
                    Stage::ProgramParsed,
                    Stage::SymbolsValidated,
                    Stage::LoweringValidated,
                    Stage::VerificationSubstrateEvaluated,
                    Stage::DeniedBeforeExecution,
                ],
                Stage::VerificationSubstrateEvaluated,
            )?
        }
    }

    Ok(())
}
