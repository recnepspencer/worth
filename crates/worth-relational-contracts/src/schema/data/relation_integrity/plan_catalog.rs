use std::collections::BTreeMap;

use crate::identity::data::KindId;

use super::LoweredRelationIntegrityPlan;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RelationIntegrityPlanCatalog {
    pub relation_plans: BTreeMap<KindId, LoweredRelationIntegrityPlan>,
}
