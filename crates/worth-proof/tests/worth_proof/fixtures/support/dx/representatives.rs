use worth_proof::{
    AssumptionBasis, AuthorityMarker, CapabilityMarker, CurrentValidity, FreshnessScopedBasis,
};

pub type CurrentBasis<B> = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>;

pub struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

pub struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

pub struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}
