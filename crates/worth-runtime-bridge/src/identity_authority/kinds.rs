use worth_foundational::facade::{FoundationalIdentityBasis, FoundationalIdentityKind};

pub struct BridgeCommitIdentityKind;
pub struct BridgeSnapshotIdentityKind;
pub struct BridgeRecordIdentityKind;
pub struct BridgePatchIdentityKind;
pub struct BridgeBranchIdentityKind;
pub struct BridgeEvidenceReferenceIdentityKind;
pub struct BridgeRetainedMappingIdentityKind;
pub struct BridgeCausalEnvelopeIdentityKind;
pub struct BridgeCausalReferenceIdentityKind;
pub struct BridgeReceiptIdentityKind;
pub struct BridgePreviewSessionIdentityKind;
pub struct BridgeWritebackDeclarationIdentityKind;

impl FoundationalIdentityKind for BridgeCommitIdentityKind {}
impl FoundationalIdentityKind for BridgeSnapshotIdentityKind {}
impl FoundationalIdentityKind for BridgeRecordIdentityKind {}
impl FoundationalIdentityKind for BridgePatchIdentityKind {}
impl FoundationalIdentityKind for BridgeBranchIdentityKind {}
impl FoundationalIdentityKind for BridgeEvidenceReferenceIdentityKind {}
impl FoundationalIdentityKind for BridgeRetainedMappingIdentityKind {}
impl FoundationalIdentityKind for BridgeCausalEnvelopeIdentityKind {}
impl FoundationalIdentityKind for BridgeCausalReferenceIdentityKind {}
impl FoundationalIdentityKind for BridgeReceiptIdentityKind {}
impl FoundationalIdentityKind for BridgePreviewSessionIdentityKind {}
impl FoundationalIdentityKind for BridgeWritebackDeclarationIdentityKind {}

pub struct BridgeCanonicalDigestIdentityBasis;
pub struct BridgeRetainedMappingDigestIdentityBasis;

impl FoundationalIdentityBasis for BridgeCanonicalDigestIdentityBasis {}
impl FoundationalIdentityBasis for BridgeRetainedMappingDigestIdentityBasis {}
