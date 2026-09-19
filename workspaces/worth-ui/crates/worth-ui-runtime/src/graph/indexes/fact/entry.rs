use crate::declaration::UiAspectName;
use crate::fact_contract::UiConsumedFactContract;

use super::{UiGraphFactConsumerIdentity, UiGraphFactConsumerKey, UiGraphFactConsumptionRelation};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGraphFactIndexEntry {
    consumer_key: UiGraphFactConsumerKey,
    consumer: UiGraphFactConsumerIdentity,
    consumption_relation: UiGraphFactConsumptionRelation,
    consumed_fact_contract: UiConsumedFactContract,
}

impl UiGraphFactIndexEntry {
    pub(crate) fn new(
        consumer_key: UiGraphFactConsumerKey,
        consumer: UiGraphFactConsumerIdentity,
        affected_aspect: Option<UiAspectName>,
        consumed_fact_contract: UiConsumedFactContract,
    ) -> Self {
        Self::new_with_relation(
            consumer_key,
            consumer,
            UiGraphFactConsumptionRelation::general(affected_aspect),
            consumed_fact_contract,
        )
    }

    pub(crate) fn new_with_relation(
        consumer_key: UiGraphFactConsumerKey,
        consumer: UiGraphFactConsumerIdentity,
        consumption_relation: UiGraphFactConsumptionRelation,
        consumed_fact_contract: UiConsumedFactContract,
    ) -> Self {
        Self {
            consumer_key,
            consumer,
            consumption_relation,
            consumed_fact_contract,
        }
    }

    pub const fn consumer_key(&self) -> &UiGraphFactConsumerKey {
        &self.consumer_key
    }

    pub const fn consumer(&self) -> UiGraphFactConsumerIdentity {
        self.consumer
    }

    pub const fn affected_aspect(&self) -> Option<&UiAspectName> {
        self.consumption_relation.affected_aspect()
    }

    pub(crate) const fn consumption_relation(&self) -> &UiGraphFactConsumptionRelation {
        &self.consumption_relation
    }

    pub const fn consumed_fact_contract(&self) -> &UiConsumedFactContract {
        &self.consumed_fact_contract
    }
}
