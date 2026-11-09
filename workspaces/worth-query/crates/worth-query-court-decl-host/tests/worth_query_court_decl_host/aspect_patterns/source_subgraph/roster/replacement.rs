//! Replacement has separate authored Create/Retire roles for the two graph occurrences.
use super::*;
pub struct ReplacementCreated;
pub struct ReplacementRetired;
pub struct Restored;
pub struct RestoredOutputs;
pub struct ReplacementMembers;
pub struct RemovedMembers;
pub struct RestoredMembers;
macro_rules! root_role {
    ($role:ident, $contract:ty, $action:ty, $cardinality:ty, $name:literal) => {
        impl WorthQueryApplicationOutputRole for $role {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = Source;
            type Action = $action;
            type Cardinality = $cardinality;
            const NAME: &'static str = $name;
        }
    };
}
root_role!(
    ReplacementCreated,
    RetiredOutputs,
    WorthQueryCreateOutput,
    WorthQueryAtMostOneOutput,
    "replacement-root"
);
root_role!(
    ReplacementRetired,
    RestoredOutputs,
    WorthQueryRetireOutput,
    WorthQueryAtMostOneOutput,
    "remove-replacement-root"
);
root_role!(
    Restored,
    RestoredOutputs,
    WorthQueryCreateOutput,
    WorthQueryExactlyOneOutput,
    "restored-root"
);
macro_rules! family {
    ($family:ident, $contract:ty, $posture:ident, $prefix:literal) => {
        impl WorthQueryApplicationOutputRoleFamily for $family {
            type Schema = Schema;
            type Contract = $contract;
            type Entity = Member;
            const PREFIX: &'static str = $prefix;
            const POSTURES: ApplicationMutationOutputPostureSet =
                ApplicationMutationOutputPostureSet::$posture;
            const MINIMUM: usize = 0;
        }
    };
}
family!(
    ReplacementMembers,
    RetiredOutputs,
    CREATE,
    "replacement-member."
);
family!(
    RemovedMembers,
    RestoredOutputs,
    RETIRE,
    "remove-replacement-member."
);
family!(RestoredMembers, RestoredOutputs, CREATE, "restored-member.");
impl ApplicationMutationOutputContract<Schema> for RestoredOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[Restored::DESCRIPTOR, ReplacementRetired::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[RestoredMembers::DESCRIPTOR, RemovedMembers::DESCRIPTOR];
}
pub struct Restoration {
    pub scope: AnchorTarget,
    pub remove: Option<Retirement>,
}
