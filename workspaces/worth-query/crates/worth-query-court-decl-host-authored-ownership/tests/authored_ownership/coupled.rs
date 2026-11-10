//! A body owns its output; the authored stage is a linked input, never its member.
use super::*;
mod co_anchors;
mod courts;
pub(super) mod history;
pub(super) mod multi;
pub(super) mod producer;
mod single_round;

impl<S: Scenario> WorthQueryApplicationRequiredOutputSource<S, program::Start>
    for lifecycle::RetireBinding<S>
{
    fn demand_from_source(
        _: &lifecycle::Retire,
    ) -> Result<program::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Err(WorthQueryRequiredOutputConnectionDenial::new(
            "stage FeatureBody pre-image is required",
        ))
    }
    fn demand_from_preimage(
        _: &lifecycle::Retire,
        preimage: &dyn ApplicationCorrectionPreImage<S>,
    ) -> Result<program::Demand, WorthQueryRequiredOutputConnectionDenial> {
        let locus = DeclaredPreImageLocus::from_field(schema::value::<S>());
        let role = <lifecycle::Retired<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR;
        let value = preimage
            .field_for_output(&role, &locus)
            .ok_or_else(|| WorthQueryRequiredOutputConnectionDenial::new("stage FeatureBody"))?;
        let key = U64ApplicationValueBinding::decode(value)
            .map_err(|_| WorthQueryRequiredOutputConnectionDenial::new("stage BodyKey"))?;
        Ok(program::Demand(key))
    }
}
impl<S: Scenario> WorthQueryApplicationRequiredOutputSource<S, program::Start>
    for lifecycle::RestoreBinding<S>
{
    fn demand_from_source(
        input: &lifecycle::Restore,
    ) -> Result<program::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(program::Demand(input.value))
    }
}

mod component_reads;

mod fresh_subject;

mod history_seams;
mod owner_set_history;

mod append;

impl<S: Scenario> WorthQueryApplicationRequiredOutputSource<S, program::Start>
    for lifecycle::CreateBinding<S>
{
    fn demand_from_source(
        input: &lifecycle::Create,
    ) -> Result<program::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(program::Demand(input.scope))
    }
}

mod cross_route;

mod foreign_participant;

mod private_membership;

mod foreign_walk;
