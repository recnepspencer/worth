use super::authority::RelationalSourceTruthAuthority;
use worth_foundational::facade::FoundationalAuthorityIdentity;

pub type RelationalSourceTruthAuthorityIdentity<Value, Kind> =
    FoundationalAuthorityIdentity<Value, RelationalSourceTruthAuthority, Kind>;
