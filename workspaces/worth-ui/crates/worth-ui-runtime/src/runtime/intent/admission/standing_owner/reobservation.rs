use super::UiIntentOperabilityStandingOwner;
use crate::runtime::intent::operability::UiIntentOperabilityDecision;
use crate::runtime::intent::operability::UiIntentOperabilityStandingFact;

impl UiIntentOperabilityStandingOwner {
    /// Refreshes the existing facts of `routes` whose decision `observe`
    /// re-derives differently. It never creates a fact: a route or instance
    /// without one is skipped, and `observe` returning `None` leaves the fact
    /// to its lifecycle. Returns how many existing facts were attempted.
    pub(in crate::runtime::intent::admission) fn reobserve<'route>(
        &mut self,
        routes: impl IntoIterator<Item = &'route str>,
        mut observe: impl FnMut(&UiIntentOperabilityStandingFact) -> Option<UiIntentOperabilityDecision>,
    ) -> u64 {
        let mut attempted = 0u64;
        for route in routes {
            let Some(members) = self.routes.get(route).cloned() else {
                continue;
            };
            for instance in members.iter() {
                let Some(row) = self.facts.get(instance) else {
                    continue;
                };
                let Some(fact) = row.routes.get(route) else {
                    continue;
                };
                attempted = attempted
                    .checked_add(1)
                    .expect("bounded standing re-observation accounting exhausted");
                let Some(decision) = observe(fact) else {
                    continue;
                };
                if *fact.decision() == decision {
                    continue;
                }
                let revision = self.next_revision();
                let refreshed = fact.with_decision(decision, revision);
                let mut row = row.clone();
                row.routes.insert(route.into(), refreshed);
                self.facts.insert(*instance, row);
                self.revision = revision;
            }
        }
        attempted
    }

    #[cfg(test)]
    pub(super) fn route_members(&self, route: &str) -> usize {
        self.routes.get(route).map_or(0, |members| members.len())
    }
}
