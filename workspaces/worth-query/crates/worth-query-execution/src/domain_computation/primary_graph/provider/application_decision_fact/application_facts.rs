//! Descriptive projection of checked application facts, without session authority.
use super::WorthQueryPrimaryGraphApplicationDecisionFact as Decision;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_checkpoint::facts::FactPayload,
};
#[cfg(test)]
use std::sync::Arc;
use std::{collections::BTreeMap, ops::Deref};
use worth_query_installation::facade::WorthQueryOperationGraphReadScope;

/// Keeps exact backing and its native/decode funding; never a freshness proof.
#[derive(Clone)]
pub(in crate::domain_computation) struct SharedObservedFact {
    payload: FactPayload,
    ordinal: usize,
}
impl AsRef<Fact> for SharedObservedFact {
    fn as_ref(&self) -> &Fact {
        self.payload
            .get(self.ordinal)
            .expect("completed fact ordinal")
    }
}
impl Deref for SharedObservedFact {
    type Target = Fact;
    fn deref(&self) -> &Fact {
        self.as_ref()
    }
}

#[derive(Clone)]
pub(in crate::domain_computation) struct SharedReadScope {
    backing: ScopeBacking,
}
#[derive(Clone)]
enum ScopeBacking {
    Installed {
        installed:
            crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
        ordinal: usize,
    },
    #[cfg(test)]
    OwnerFixture(Arc<WorthQueryOperationGraphReadScope>),
}
impl AsRef<WorthQueryOperationGraphReadScope> for SharedReadScope {
    fn as_ref(&self) -> &WorthQueryOperationGraphReadScope {
        match &self.backing {
            ScopeBacking::Installed { installed, ordinal } => installed
                .get(*ordinal)
                .expect("completed installed scope ordinal"),
            #[cfg(test)]
            ScopeBacking::OwnerFixture(scope) => scope,
        }
    }
}

pub(in crate::domain_computation) struct CompletedApplicationFacts {
    installed: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
    original: FactPayload,
}
impl CompletedApplicationFacts {
    pub(in crate::domain_computation::primary_graph) fn new(
        installed: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
        original: FactPayload,
    ) -> Self {
        Self {
            installed,
            original,
        }
    }

    /// This remains the full cold binding path. Authorization appends only its
    /// newly issued principal/requirement observations after this projection.
    pub(in crate::domain_computation) fn into_decisions(
        self,
    ) -> Result<Vec<Decision>, &'static str> {
        if self.installed.len() > self.original.len() {
            return Err("installed read scopes exceed retained application facts");
        }
        let installed_count = self.installed.len();
        let mut facts: Vec<_> = (0..installed_count)
            .map(|ordinal| {
                Decision::application(
                    SharedReadScope {
                        backing: ScopeBacking::Installed {
                            installed: self.installed.clone(),
                            ordinal,
                        },
                    },
                    SharedObservedFact {
                        payload: self.original.clone(),
                        ordinal,
                    },
                )
            })
            .collect();
        let mut indices = facts
            .iter()
            .enumerate()
            .map(|(index, fact)| (fact.locator_identity(), index))
            .collect::<BTreeMap<_, _>>();
        if indices.len() != facts.len() {
            return Err("application decision facts contain duplicate structural locators");
        }
        for ordinal in installed_count..self.original.len() {
            let source = SharedObservedFact {
                payload: self.original.clone(),
                ordinal,
            };
            let locator = source.dependency_locator_identity();
            if let Some(index) = indices.get(&locator).copied() {
                facts[index] = facts[index].clone().retain_observed_source_role(source)?;
            } else {
                indices.insert(locator, facts.len());
                facts.push(Decision::observed_source(source));
            }
        }
        Ok(facts)
    }
}

#[cfg(test)]
impl From<Fact> for SharedObservedFact {
    fn from(fact: Fact) -> Self {
        Self {
            payload: std::sync::Arc::<[Fact]>::from([fact]).into(),
            ordinal: 0,
        }
    }
}

#[cfg(test)]
impl From<WorthQueryOperationGraphReadScope> for SharedReadScope {
    fn from(scope: WorthQueryOperationGraphReadScope) -> Self {
        Self {
            backing: ScopeBacking::OwnerFixture(Arc::new(scope)),
        }
    }
}
