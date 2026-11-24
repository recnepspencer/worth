use worth_foundational::facade::{AspectValue, CanonicalDigestId};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryObservableInfluence, ApplicationQueryResultSlotKey,
};
use worth_query_execution_lineage::facade::ports::ApplicationDisclosureReceiptInternalPort as _;
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkEvidence, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::identity::EntityId;

use super::super::WorthQueryApplicationDisclosureReceipt;
use super::{
    WorthQueryApplicationDisclosedProjectionAdmission,
    WorthQueryApplicationInternalProjectionAdmission, WorthQueryApplicationQueryGovernance,
    WorthQueryPendingApplicationQueryGovernance,
};
use crate::domain_computation::authorization::WorthQueryRetainedCapabilityAuthorization;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::provider_session::{
    WorthQueryGraphWorkSessionIdentity, WorthQueryManagedGraphWorkSession,
};
impl WorthQueryApplicationQueryGovernance {
    /// Return the genuine capability request to admission. Session-bound
    /// computation and disclosure decisions must be issued again by that owner.
    pub fn into_readmission(self) -> Option<WorthQueryPendingApplicationQueryGovernance> {
        match self {
            Self::Public => None,
            Self::Governed {
                capability_name,
                capability_type,
                disclosure_value,
                authorization_canonical_work,
                authorization,
                ..
            } => Some(WorthQueryPendingApplicationQueryGovernance::new(
                capability_name,
                capability_type,
                disclosure_value,
                authorization_canonical_work,
                authorization,
            )),
        }
    }

    pub fn into_pending(self) -> Option<WorthQueryPendingApplicationQueryGovernance> {
        match self {
            Self::Public => None,
            Self::Governed {
                capability_name,
                capability_type,
                disclosure_value,
                authorization_canonical_work,
                authorization,
                ..
            } => Some(WorthQueryPendingApplicationQueryGovernance {
                capability_name,
                capability_type,
                disclosure_value,
                authorization_canonical_work,
                authorization,
            }),
        }
    }

    pub fn is_disclosed(&self, slot: &ApplicationQueryResultSlotKey) -> bool {
        match self {
            Self::Public => true,
            Self::Governed { disclosure, .. } => disclosure.disclosed.contains_key(slot),
        }
    }

    pub fn omission(&self, slot: &ApplicationQueryResultSlotKey) -> Option<(&str, &AspectValue)> {
        match self {
            Self::Public => None,
            Self::Governed { disclosure, .. } => disclosure
                .omitted
                .get(slot)
                .map(|rule| (disclosure.classification.as_str(), rule.disclosure_value())),
        }
    }

    pub fn admit_disclosed_projection(
        &self,
        slot: &ApplicationQueryResultSlotKey,
        field: (&str, &str, &str),
        field_key: &worth_foundational::facade::FieldKey,
    ) -> Option<WorthQueryApplicationDisclosedProjectionAdmission> {
        match self {
            Self::Public => Some(WorthQueryApplicationDisclosedProjectionAdmission::public()),
            Self::Governed { disclosure, .. } => {
                let admitted = disclosure.disclosed.get(slot)?.field()?;
                admitted.matches(field).then_some(())?;
                WorthQueryApplicationDisclosedProjectionAdmission::governed(admitted, field_key)
            }
        }
    }

    pub fn internal_rule_count(&self) -> usize {
        match self {
            Self::Public => 0,
            Self::Governed { computation, .. } => computation
                .internal_field_rules
                .len()
                .saturating_add(computation.internal_relation_rules.len()),
        }
    }

    pub fn admit_internal_projection<'a>(
        &'a self,
        field: (&str, &str, &str),
        field_key: &'a worth_foundational::facade::FieldKey,
        surface: ApplicationQueryObservableInfluence,
    ) -> Option<WorthQueryApplicationInternalProjectionAdmission<'a>> {
        match self {
            Self::Public => Some(WorthQueryApplicationInternalProjectionAdmission::public(
                field_key,
            )),
            Self::Governed { computation, .. } => {
                let index = computation
                    .internal_field_rules
                    .binary_search_by(|((e, a, f), _)| {
                        (e.as_str(), a.as_str(), f.as_str()).cmp(&field)
                    })
                    .ok()?;
                let (key, group) = &computation.internal_field_rules[index];
                if !group.permits(surface) {
                    return None;
                }
                let field = group
                    .field()
                    .filter(|field| field.matches(field_tuple(key)))?;
                WorthQueryApplicationInternalProjectionAdmission::governed(field, field_key)
            }
        }
    }

    pub fn admit_internal_relation(
        &self,
        relation: (&str, &str, &str),
        direction: worth_query_declaration::facade::application_query::RelationDirection,
        surface: ApplicationQueryObservableInfluence,
    ) -> bool {
        match self {
            Self::Public => true,
            Self::Governed { computation, .. } => {
                let target = (relation.0, relation.1, relation.2, direction);
                let Ok(index) =
                    computation
                        .internal_relation_rules
                        .binary_search_by(|((r, f, t, d), _)| {
                            (r.as_str(), f.as_str(), t.as_str(), *d).cmp(&target)
                        })
                else {
                    return false;
                };
                computation.internal_relation_rules[index]
                    .1
                    .permits(surface)
            }
        }
    }

    pub fn authorization_mut(&mut self) -> Option<&mut WorthQueryRetainedCapabilityAuthorization> {
        match self {
            Self::Public => None,
            Self::Governed { authorization, .. } => Some(authorization),
        }
    }

    pub fn authorization(&self) -> Option<&WorthQueryRetainedCapabilityAuthorization> {
        match self {
            Self::Public => None,
            Self::Governed { authorization, .. } => Some(authorization),
        }
    }

    pub const fn authorization_canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        match self {
            Self::Public => WorthQueryCanonicalWorkEvidence::zero(),
            Self::Governed {
                authorization_canonical_work,
                ..
            } => *authorization_canonical_work,
        }
    }

    pub fn authorization_belongs_to_session(
        &self,
        session: WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        self.authorization()
            .is_none_or(|authorization| authorization.belongs_to_session(session))
    }

    pub fn receipt(&self) -> WorthQueryApplicationDisclosureReceipt {
        match self {
            Self::Public => WorthQueryApplicationDisclosureReceipt::public(),
            Self::Governed {
                disclosure,
                authorization,
                ..
            } => WorthQueryApplicationDisclosureReceipt::governed(
                disclosure.classification.clone(),
                disclosure
                    .disclosed
                    .iter()
                    .map(|(slot, rule)| (slot.clone(), rule.disclosure_value().clone()))
                    .collect(),
                disclosure
                    .omitted
                    .iter()
                    .map(|(slot, rule)| (slot.clone(), rule.disclosure_value().clone()))
                    .collect(),
                authorization.capability_authority_identity(),
                authorization.decision_identity(),
                authorization.exact_fact_count(),
            ),
        }
    }

    pub fn computation_matches(
        &self,
        graph_work: &WorthQueryManagedGraphWorkSession,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        match self {
            Self::Public => true,
            Self::Governed { computation, .. } => {
                computation
                    .binding
                    .matches(graph_work, runtime, query, parameters, principal, scope)
                    && !matches!(computation.disclosure_value, AspectValue::Null)
            }
        }
    }

    pub fn readmission_matches(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        parameters: &CanonicalDigestId,
        principal: EntityId,
        scope: EntityId,
    ) -> bool {
        match self {
            Self::Public => true,
            Self::Governed { computation, .. } => computation
                .binding
                .readmission_matches(runtime, query, parameters, principal, scope),
        }
    }
}

fn field_tuple(field: &(String, String, String)) -> (&str, &str, &str) {
    (&field.0, &field.1, &field.2)
}

mod court_doors_for_test;
