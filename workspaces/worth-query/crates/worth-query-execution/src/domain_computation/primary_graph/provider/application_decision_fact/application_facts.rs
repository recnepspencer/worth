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
impl SharedObservedFact {
    pub(super) fn at(payload: FactPayload, ordinal: usize) -> Self {
        Self { payload, ordinal }
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
impl SharedReadScope {
    pub(super) fn at(
        installed: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
        ordinal: usize,
    ) -> Self {
        Self {
            backing: ScopeBacking::Installed { installed, ordinal },
        }
    }
}

pub(in crate::domain_computation) struct CompletedApplicationFacts {
    installed: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
    original: FactPayload,
    binding_memory: Option<worth_execution::ExecutionMemoryReservation>,
    binding_bytes: Option<u64>,
    projection: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::CompletedFactProjection,
        >,
    >,
    original_read: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::CompletedOriginalRead,
        >,
    >,
}
impl CompletedApplicationFacts {
    pub(in crate::domain_computation::primary_graph) fn new(
        installed: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
        original: FactPayload,
    ) -> Self {
        Self {
            installed,
            original,
            binding_memory: None,
            binding_bytes: None,
            projection: None,
            original_read: None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn with_completed_projection(
        mut self,
        projection: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CompletedFactProjection>>,
        original_read: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CompletedOriginalRead>>,
    ) -> Self {
        self.projection = projection;
        self.original_read = original_read;
        self
    }
    pub(in crate::domain_computation::primary_graph) fn admit_binding(
        mut self,
        control: crate::domain_computation::primary_graph::StorageControl<'_, '_>,
        authorization_count: usize,
    ) -> Result<Self, crate::domain_computation::WorthQueryDecisionReadSetFailure> {
        use crate::domain_computation::{
            WorthQueryDecisionReadSetDenialKind as Kind,
            WorthQueryDecisionReadSetFailure as Failure,
        };
        let denied = || Failure::new(Kind::FactCountOverflow, "application projection capacity");
        let first = if self.projection.is_some() {
            self.installed.len()
        } else {
            0
        };
        let node = (11 * std::mem::size_of::<(String, Decision)>()
            + 16 * std::mem::size_of::<usize>()) as u64;
        let entry = node.checked_add(2048).ok_or_else(denied)?;
        let mut bytes = (authorization_count as u64)
            .checked_mul(entry)
            .and_then(|n| {
                n.checked_add(
                    (std::mem::size_of::<super::ApplicationFactTable>()
                        + 2 * std::mem::size_of::<usize>()) as u64,
                )
            })
            .ok_or_else(denied)?;
        for ordinal in first..self.original.len() {
            control
                .check_live()
                .map_err(super::super::decision_facts::storage_read_set_failure)?;
            crate::domain_computation::primary_graph::fact_verification_observation::provider_application_fact();
            let fact = self.original.get(ordinal).ok_or_else(denied)?;
            bytes = bytes
                .checked_add(
                    fact.dependency_locator_byte_bound()
                        .ok_or_else(denied)?
                        .checked_mul(16)
                        .and_then(|n| n.checked_add(entry))
                        .ok_or_else(denied)?,
                )
                .ok_or_else(denied)?;
        }
        let memory = control.policy().reserve_memory(bytes);
        control
            .check_live()
            .map_err(super::super::decision_facts::storage_read_set_failure)?;
        self.binding_memory = memory.map_err(Failure::allocation_denied)?;
        self.binding_bytes = Some(bytes);
        Ok(self)
    }
    pub(in crate::domain_computation) fn into_table(
        self,
    ) -> Result<super::ApplicationFactTable, &'static str> {
        let bytes = self
            .binding_bytes
            .ok_or("application binding was not admitted")?;
        if let Some(projection) = self.projection.clone() {
            super::ApplicationFactTable::projected(
                self.original,
                self.installed,
                projection,
                self.original_read,
                self.binding_memory,
                bytes,
            )
        } else {
            let mut this = self;
            let memory = this.binding_memory.take();
            super::ApplicationFactTable::full(this.into_decisions()?, memory, bytes)
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
                crate::domain_computation::primary_graph::fact_verification_observation::provider_application_fact();
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
            crate::domain_computation::primary_graph::fact_verification_observation::provider_application_fact();
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
