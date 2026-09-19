use std::num::NonZeroU64;

use worth_query_decl::facade::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, RelationalInvariantWorkBudget,
};

use super::WorthUiApplicationSchema;

pub struct WorthUiStatusIntegrity;

impl ApplicationInvariantMarkerIdentity<WorthUiApplicationSchema> for WorthUiStatusIntegrity {
    const IDENTIFIER: &'static str = "WorthUiStatusIntegrity";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub(crate) fn status_integrity_invariant(
) -> ApplicationInvariantDefinition<WorthUiApplicationSchema, WorthUiStatusIntegrity> {
    ApplicationInvariantDefinition::new(
        WorthUiStatusIntegrity::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(NonZeroU64::new(64).expect("non-zero UI work budget")),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity(
                "WorthUiRecord".to_owned(),
            )],
            [ApplicationInvariantScopeTarget::Entity(
                "WorthUiRecord".to_owned(),
            )],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
