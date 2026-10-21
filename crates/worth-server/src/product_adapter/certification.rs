#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerProductAdapterCertificationCode {
    BlankAdapterLabel,
    MissingDeclarations,
    BlankOperationName,
    BlankPayloadSchemaIdentity,
    BlankSupportSnapshotRow,
    MissingErrorMap,
    MissingQueryApplicationReadinessProvider,
    BlankDraftScope,
    BlankCoordinationLane,
    MissingDurableMutationExecutor,
    IncompatibleDurableMutationCapability,
    IncompatibleRetryOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductAdapterCertificationError {
    code: WorthServerProductAdapterCertificationCode,
    detail: String,
}

impl WorthServerProductAdapterCertificationError {
    pub(crate) fn new(
        code: WorthServerProductAdapterCertificationCode,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub fn code(&self) -> WorthServerProductAdapterCertificationCode {
        self.code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
