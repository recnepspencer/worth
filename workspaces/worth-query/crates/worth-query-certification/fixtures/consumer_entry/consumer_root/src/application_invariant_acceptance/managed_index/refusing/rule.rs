//! This fixed application declares default Refuse from its first installation.
use super::{prefix, schema::*};
use worth_query_decl::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::*,
};

pub(super) struct ReservedName;
impl ApplicationInvariantMarkerIdentity<Catalog> for ReservedName {
    const IDENTIFIER: &'static str = "AtMostOneTakenName";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition() -> ApplicationInvariantDefinition<Catalog, ReservedName> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Directory".into()),
        ApplicationInvariantScopeTarget::Entity("Item".into()),
        ApplicationInvariantScopeTarget::Relation("Contains".into()),
    ];
    ApplicationInvariantDefinition::new(
        ReservedName::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            closure.clone(),
            closure,
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
    .managed_index_source(prefix::PrefixIndex::reference())
}

mod evaluation;
pub(super) use evaluation::{resolve, Calls};
