/// Internal meaning that a framed selection artifact failed to establish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRetainedSelectionArtifactDenial {
    SourceDescriptor,
    EntityIdentity,
    SchemaAgreement,
    TargetPredecessor,
    UnsupportedProfile,
    SelectedRecordMeaning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalCommitArtifactDenial {
    Parentage,
    RootLinkage,
    CanonicalPayloadEncoding,
    RetainedEntitySelection(RelationalRetainedSelectionArtifactDenial),
}
