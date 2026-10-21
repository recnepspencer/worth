//! Current-body views over checked ordinals and a freshly admitted overlay.
use super::{application_facts::*, WorthQueryPrimaryGraphApplicationDecisionFact as Decision};
use crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes;
use crate::domain_computation::primary_graph::application_attempt::{
    CompletedFactProjection, CompletedOriginalRead,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use std::{collections::BTreeMap, sync::Arc};

mod preimage;

pub(in crate::domain_computation) struct ApplicationFactTable {
    prefix: Option<Prefix>,
    overlay: BTreeMap<String, Decision>,
    count: usize,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    binding_bytes: u64,
}
struct Prefix {
    body: FactPayload,
    scopes: CompletedReadScopes,
    projection: Arc<CompletedFactProjection>,
    original: Option<Arc<CompletedOriginalRead>>,
}
impl ApplicationFactTable {
    pub(in crate::domain_computation) fn full_capture_bytes(&self) -> Option<u64> {
        self.prefix
            .as_ref()
            .map_or(Some(0), |p| p.projection.bytes().checked_mul(4))
            .and_then(|n| n.checked_add(self.binding_bytes))
    }
    pub(in crate::domain_computation) fn requests(
        &self,
    ) -> impl Iterator<Item = crate::domain_computation::WorthQueryDecisionFactRequest> + '_ {
        self.values().map(|fact| {
            crate::domain_computation::primary_graph::fact_verification_observation::provider_application_fact();
            request(&fact)
        })
    }
    pub(in crate::domain_computation) fn explicit_requests(
        &self,
    ) -> impl Iterator<Item = crate::domain_computation::WorthQueryDecisionFactRequest> + '_ {
        self.explicit_values().map(request)
    }
    pub(super) fn full(
        facts: Vec<Decision>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        binding_bytes: u64,
    ) -> Result<Self, &'static str> {
        let count = facts.len();
        let overlay: BTreeMap<_, _> = facts
            .into_iter()
            .map(|f| {
                let key = f.locator_identity();
                validate_locator(&key)?;
                Ok((key, f))
            })
            .collect::<Result<_, &'static str>>()?;
        if count != overlay.len() {
            return Err("duplicate decision locators");
        }
        Ok(Self {
            prefix: None,
            overlay,
            count,
            _memory: memory,
            binding_bytes,
        })
    }
    pub(super) fn projected(
        body: FactPayload,
        scopes: CompletedReadScopes,
        projection: Arc<CompletedFactProjection>,
        original: Option<Arc<CompletedOriginalRead>>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        binding_bytes: u64,
    ) -> Result<Self, &'static str> {
        if projection.count() != scopes.len() || scopes.len() > body.len() {
            return Err("completed projection does not cover exact installed scopes");
        }
        let count = scopes.len();
        let mut table = Self {
            prefix: Some(Prefix {
                body,
                scopes,
                projection,
                original,
            }),
            overlay: BTreeMap::new(),
            count,
            _memory: memory,
            binding_bytes,
        };
        let prefix = table.prefix.as_ref().expect("projected table");
        let sources: Vec<_> = (count..prefix.body.len())
            .map(|ordinal| SharedObservedFact::at(prefix.body.clone(), ordinal))
            .collect();
        for source in sources {
            crate::domain_computation::primary_graph::fact_verification_observation::provider_application_fact();
            let key = source.dependency_locator_identity();
            validate_locator(&key)?;
            let fact = match table.get(&key) {
                Some(fact) => fact.retain_observed_source_role(source)?,
                None => {
                    table.count += 1;
                    Decision::observed_source(source)
                }
            };
            table.overlay.insert(key, fact);
        }
        Ok(table)
    }
    pub(in crate::domain_computation) fn insert_fresh(
        &mut self,
        fact: Decision,
    ) -> Result<(), &'static str> {
        let key = fact.locator_identity();
        validate_locator(&key)?;
        if self.contains_key(&key) {
            return Err("duplicate fresh decision locator");
        }
        self.overlay.insert(key, fact);
        self.count += 1;
        Ok(())
    }
    pub(in crate::domain_computation) fn len(&self) -> usize {
        self.count
    }
    pub(in crate::domain_computation) fn contains_key(&self, key: &str) -> bool {
        self.overlay.contains_key(key)
            || self
                .prefix
                .as_ref()
                .is_some_and(|p| p.projection.ordinal(key).is_some())
    }
    pub(in crate::domain_computation) fn get(&self, key: &str) -> Option<Decision> {
        self.overlay.get(key).cloned().or_else(|| {
            let prefix = self.prefix.as_ref()?;
            let ordinal = prefix.projection.ordinal(key)?;
            Some(Decision::application(
                SharedReadScope::at(prefix.scopes.clone(), ordinal),
                SharedObservedFact::at(prefix.body.clone(), ordinal),
            ))
        })
    }
    pub(in crate::domain_computation) fn values(&self) -> impl Iterator<Item = Decision> + '_ {
        let mut prefix = self
            .prefix
            .iter()
            .flat_map(|p| p.projection.locators())
            .peekable();
        let mut overlay = self.overlay.iter().peekable();
        std::iter::from_fn(move || {
            let take_prefix = match (prefix.peek(), overlay.peek()) {
                (Some((a, _)), Some((b, _))) if *a < b.as_str() => true,
                (Some((a, _)), Some((b, _))) if *a == b.as_str() => {
                    prefix.next();
                    return overlay.next().map(|(_, fact)| fact.clone());
                }
                (_, Some(_)) => return overlay.next().map(|(_, fact)| fact.clone()),
                (Some(_), None) => true,
                (None, None) => return None,
            };
            if take_prefix {
                let (_, ordinal) = prefix.next()?;
                let p = self.prefix.as_ref()?;
                Some(Decision::application(
                    SharedReadScope::at(p.scopes.clone(), ordinal),
                    SharedObservedFact::at(p.body.clone(), ordinal),
                ))
            } else {
                None
            }
        })
    }
    pub(in crate::domain_computation) fn explicit_values(&self) -> impl Iterator<Item = &Decision> {
        self.overlay.values()
    }
    pub(in crate::domain_computation) fn closed_prefix(
        &self,
    ) -> Option<(&Arc<CompletedFactProjection>, &Arc<CompletedOriginalRead>)> {
        let prefix = self.prefix.as_ref()?;
        let original = prefix.original.as_ref()?;
        original
            .matches_body(&prefix.body, &prefix.projection)
            .then_some((&prefix.projection, original))
    }
    pub(in crate::domain_computation) fn grouped_count(&self) -> usize {
        self.prefix.as_ref().map_or(0, |prefix| {
            prefix.projection.count()
                - self
                    .overlay
                    .keys()
                    .filter(|key| prefix.projection.ordinal(key).is_some())
                    .count()
        })
    }
}

fn validate_locator(key: &str) -> Result<(), &'static str> {
    crate::domain_computation::WorthQueryDecisionFactLocator::structural_proof(key.to_owned())
        .map(|_| ())
        .map_err(|_| "invalid completed structural locator")
}

fn request(fact: &Decision) -> crate::domain_computation::WorthQueryDecisionFactRequest {
    use crate::domain_computation::{WorthQueryDecisionFactLocator, WorthQueryDecisionFactRequest};
    use worth_query_installation::facade::{
        APPLICATION_AUTHORIZATION_FACT_FAMILY, APPLICATION_DECISION_FACT_FAMILY,
    };
    let family = if fact.is_application_decision() {
        APPLICATION_DECISION_FACT_FAMILY
    } else {
        APPLICATION_AUTHORIZATION_FACT_FAMILY
    };
    let locator = WorthQueryDecisionFactLocator::structural_proof(fact.locator_identity())
        .expect("completed native or freshly authorized structural locator");
    WorthQueryDecisionFactRequest::new(family, locator).expect("installed native decision family")
}
