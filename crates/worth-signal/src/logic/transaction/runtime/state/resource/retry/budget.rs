use crate::data::resource::{
    InFlightResourceRequest, ResourceGeneration, ResourceNodeId, ResourceRetryBudgetScope,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::logic::transaction::runtime) struct ResourceRetryBudgetCharge {
    scope: ResourceRetryBudgetScope,
    limit: u32,
    spent_before: u32,
}

impl ResourceRetryBudgetCharge {
    pub(in crate::logic::transaction::runtime) fn scope(self) -> ResourceRetryBudgetScope {
        self.scope
    }

    pub(in crate::logic::transaction::runtime) fn limit(self) -> u32 {
        self.limit
    }

    pub(in crate::logic::transaction::runtime) fn spent_before(self) -> u32 {
        self.spent_before
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(in crate::logic::transaction::runtime::state::resource) struct ResourceRetryBudgetLedger {
    spend_by_generation: crate::data::persistent_ord_map::PersistentOrdMap<ResourceGeneration, u32>,
    spend_by_node: crate::data::persistent_ord_map::PersistentOrdMap<ResourceNodeId, u32>,
    runtime_spend: u32,
}

impl ResourceRetryBudgetLedger {
    pub(in crate::logic::transaction::runtime) fn fork_persistent(&mut self) -> Self {
        Self {
            spend_by_generation: self.spend_by_generation.fork_persistent(),
            spend_by_node: self.spend_by_node.fork_persistent(),
            runtime_spend: self.runtime_spend,
        }
    }

    #[cfg(test)]
    pub(in crate::logic::transaction::runtime) fn fork_storage_identity(&self) -> Self {
        Self {
            spend_by_generation: self.spend_by_generation.fork_storage_identity(),
            spend_by_node: self.spend_by_node.fork_storage_identity(),
            runtime_spend: self.runtime_spend,
        }
    }

    #[cfg(test)]
    pub(in crate::logic::transaction::runtime) fn shares_storage_with(&self, other: &Self) -> bool {
        self.spend_by_generation.ptr_eq(&other.spend_by_generation)
            && self.spend_by_node.ptr_eq(&other.spend_by_node)
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn charge_for(
        &self,
        in_flight: &InFlightResourceRequest,
        retry_budget_scope: Option<ResourceRetryBudgetScope>,
        retry_budget_limit: Option<u32>,
    ) -> Option<ResourceRetryBudgetCharge> {
        let (scope, limit) = match (retry_budget_scope, retry_budget_limit) {
            (Some(scope), Some(limit)) => (scope, limit),
            _ => return None,
        };
        let spent_before = match scope {
            ResourceRetryBudgetScope::Request => self
                .spend_by_generation
                .get(&in_flight.generation())
                .copied()
                .unwrap_or(0),
            ResourceRetryBudgetScope::ResourceNode => self
                .spend_by_node
                .get(&in_flight.node())
                .copied()
                .unwrap_or(0),
            ResourceRetryBudgetScope::Runtime => self.runtime_spend,
        };
        Some(ResourceRetryBudgetCharge {
            scope,
            limit,
            spent_before,
        })
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn consume(
        &mut self,
        in_flight: &InFlightResourceRequest,
        charge: ResourceRetryBudgetCharge,
    ) {
        match charge.scope {
            ResourceRetryBudgetScope::Request => {
                self.spend_by_generation.insert(
                    in_flight.generation(),
                    charge.spent_before.saturating_add(1),
                );
            }
            ResourceRetryBudgetScope::ResourceNode => {
                self.spend_by_node
                    .insert(in_flight.node(), charge.spent_before.saturating_add(1));
            }
            ResourceRetryBudgetScope::Runtime => {
                self.runtime_spend = self.runtime_spend.saturating_add(1);
            }
        }
    }

    pub(in crate::logic::transaction::runtime::state::resource) fn clear_request_generation(
        &mut self,
        generation: ResourceGeneration,
    ) {
        self.spend_by_generation.remove(&generation);
    }
}
