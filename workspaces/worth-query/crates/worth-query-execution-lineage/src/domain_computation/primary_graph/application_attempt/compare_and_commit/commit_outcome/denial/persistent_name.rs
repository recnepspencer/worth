//! Declared output names keep their typed refusal through the commit denial.
use super::{
    denial_cause::DenialCause, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
};
use worth_query_execution_naming::facade::{LineageNameDenial, WorthQueryOutputNameDenial};

impl WorthQueryApplicationCommitDenial {
    /// The operation's declared output names could not be lowered against the
    /// commit basis (a held name not live there, an unbound output, ...).
    pub fn persistent_name_output(denial: WorthQueryOutputNameDenial) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProviderRejected,
            stage: WorthQueryApplicationCommitDenialStage::ProviderPlan,
            detail: Some(denial.to_string().into()),
            cause: Some(Box::new(DenialCause::PersistentNameOutput(denial))),
        }
    }

    /// The typed refusal of the operation's declared output names, when they
    /// were refused while lowering against the commit basis.
    pub fn persistent_name_output_denial(&self) -> Option<&WorthQueryOutputNameDenial> {
        match self.cause.as_deref()? {
            DenialCause::PersistentNameOutput(denial) => Some(denial),
            _ => None,
        }
    }

    /// The naming law Relational's commit finalization refused (for example
    /// `MissingBirth`, `DuplicateBirth`, `LineagePostureMismatch` or
    /// `InvalidMergeKeeper`), proven to precede publication: zero effects.
    /// An installed invariant that read the commit's names meets the same
    /// refusal first, and the commit refuses with it.
    pub fn persistent_name_refusal(&self) -> Option<&LineageNameDenial> {
        if let Some(DenialCause::InvariantExecution(failure)) = self.cause.as_deref() {
            return failure.persistent_name_refusal();
        }
        self.native_preparation_error()?.lineage_name_denial_ref()
    }
}
