#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationSchemaContractCatalogDenialKind {
    DuplicateAspectIdentity,
    DuplicateAspectLocus,
    DuplicateFieldLocus,
    RevisionZero,
    MissingAspectFieldClosure,
    FieldWithoutAspect,
    InvalidAspectKey,
    InvalidFieldKey,
    InvalidAspectShape,
    ProjectionMaskRejected,
    CanonicalContractRejected,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationSchemaContractCatalogDenial {
    kind: WorthQueryApplicationSchemaContractCatalogDenialKind,
    subject: String,
}

impl WorthQueryApplicationSchemaContractCatalogDenial {
    pub(crate) fn new(
        kind: WorthQueryApplicationSchemaContractCatalogDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationSchemaContractCatalogDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}
