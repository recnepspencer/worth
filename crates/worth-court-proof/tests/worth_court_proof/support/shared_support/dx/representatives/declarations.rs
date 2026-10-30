pub(crate) use {
    worth_proof::AssumptionBasis, worth_proof::AuthorityMarker, worth_proof::CapabilityMarker,
    worth_proof::CurrentValidity, worth_proof::FreshnessScopedBasis,
};

pub(crate) type CurrentBasis<B> = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>;

pub(crate) struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

pub(crate) struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

pub(crate) struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}
