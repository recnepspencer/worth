use worth_foundational::facade::{
    FoundationalAuthorityIdentity, FoundationalBoundaryBridgedIdentity,
    FoundationalDigestIdentityEvidence, FoundationalExternalIdentityToken,
    FoundationalProjectionIdentity,
};

use super::authority::RelationalSourceTruthAuthority;

pub type RelationalSourceTruthAuthorityIdentity<Value, Kind> =
    FoundationalAuthorityIdentity<Value, RelationalSourceTruthAuthority, Kind>;

pub type RelationalSourceTruthBoundaryBridgedIdentity<Value, Kind> =
    FoundationalBoundaryBridgedIdentity<Value, RelationalSourceTruthAuthority, Kind>;

pub type RelationalSourceTruthExternalIdentityToken<Value, Kind> =
    FoundationalExternalIdentityToken<Value, Kind>;

pub type RelationalSourceTruthProjectionIdentity<Label, Kind> =
    FoundationalProjectionIdentity<Label, Kind>;

pub type RelationalSourceTruthDigestIdentityEvidence<Basis, Kind> =
    FoundationalDigestIdentityEvidence<Basis, RelationalSourceTruthAuthority, Kind>;
