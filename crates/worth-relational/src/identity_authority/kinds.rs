use worth_foundational::facade::{FoundationalIdentityBasis, FoundationalIdentityKind};

pub struct RelationalCommitIdentityKind;
pub struct RelationalEntityIdentityKind;
pub struct RelationalRelationIdentityKind;
pub struct RelationalSnapshotIdentityKind;
pub struct RelationalVersionIdentityKind;
pub struct RelationalBranchIdentityKind;
pub struct RelationalWorkspaceIdentityKind;

impl FoundationalIdentityKind for RelationalCommitIdentityKind {}
impl FoundationalIdentityKind for RelationalEntityIdentityKind {}
impl FoundationalIdentityKind for RelationalRelationIdentityKind {}
impl FoundationalIdentityKind for RelationalSnapshotIdentityKind {}
impl FoundationalIdentityKind for RelationalVersionIdentityKind {}
impl FoundationalIdentityKind for RelationalBranchIdentityKind {}
impl FoundationalIdentityKind for RelationalWorkspaceIdentityKind {}

pub struct RelationalCanonicalDigestIdentityBasis;

impl FoundationalIdentityBasis for RelationalCanonicalDigestIdentityBasis {}
