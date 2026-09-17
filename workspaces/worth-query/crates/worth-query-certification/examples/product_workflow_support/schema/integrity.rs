use std::num::NonZeroU64;

use worth_query_host::facade::declaration::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, RelationalInvariantWorkBudget,
};

use super::TemporalHostSchema;

pub struct TemporalIntegrity;

impl ApplicationInvariantMarkerIdentity<TemporalHostSchema> for TemporalIntegrity {
    const IDENTIFIER: &'static str = "TemporalIntegrity";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub(super) fn definition() -> ApplicationInvariantDefinition<TemporalHostSchema, TemporalIntegrity>
{
    ApplicationInvariantDefinition::new(
        TemporalIntegrity::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(NonZeroU64::new(64).expect("non-zero work budget")),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity(
                "TemporalIntent".to_owned(),
            )],
            [ApplicationInvariantScopeTarget::Entity(
                "TemporalIntent".to_owned(),
            )],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
