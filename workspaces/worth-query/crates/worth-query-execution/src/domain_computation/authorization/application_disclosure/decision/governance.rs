use worth_foundational::facade::{AspectValue, CanonicalDigestId};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryObservableInfluence, ApplicationQueryResultSlotKey,
};
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
    pub(in crate::domain_computation) fn into_pending(
        self,
    ) -> Option<WorthQueryPendingApplicationQueryGovernance> {
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

    pub(in crate::domain_computation) fn is_disclosed(
        &self,
        slot: &ApplicationQueryResultSlotKey,
    ) -> bool {
        match self {
            Self::Public => true,
            Self::Governed { disclosure, .. } => disclosure.disclosed.contains_key(slot),
        }
    }

    pub(in crate::domain_computation) fn omission(
        &self,
        slot: &ApplicationQueryResultSlotKey,
    ) -> Option<(&str, &AspectValue)> {
        match self {
            Self::Public => None,
            Self::Governed { disclosure, .. } => disclosure
                .omitted
                .get(slot)
                .map(|rule| (disclosure.classification.as_str(), rule.disclosure_value())),
        }
    }

    pub(in crate::domain_computation) fn admit_disclosed_projection(
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

    pub(in crate::domain_computation) fn admit_internal_projection<'a>(
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
                let key = (
                    field.0.to_string(),
                    field.1.to_string(),
                    field.2.to_string(),
                );
                let rules = computation.internal_field_rules.get(&key)?;
                let admitted = rules.iter().all(|rule| rule.influence().permits(surface));
                let field = rules
                    .first()?
                    .field()
                    .filter(|field| field.matches(field_tuple(&key)))?;
                admitted.then_some(())?;
                WorthQueryApplicationInternalProjectionAdmission::governed(field, field_key)
            }
        }
    }

    pub(in crate::domain_computation) fn authorization_mut(
        &mut self,
    ) -> Option<&mut WorthQueryRetainedCapabilityAuthorization> {
        match self {
            Self::Public => None,
            Self::Governed { authorization, .. } => Some(authorization),
        }
    }

    pub(in crate::domain_computation) fn authorization(
        &self,
    ) -> Option<&WorthQueryRetainedCapabilityAuthorization> {
        match self {
            Self::Public => None,
            Self::Governed { authorization, .. } => Some(authorization),
        }
    }

    pub(in crate::domain_computation) const fn authorization_canonical_work(
        &self,
    ) -> WorthQueryCanonicalWorkEvidence {
        match self {
            Self::Public => WorthQueryCanonicalWorkEvidence::zero(),
            Self::Governed {
                authorization_canonical_work,
                ..
            } => *authorization_canonical_work,
        }
    }

    pub(in crate::domain_computation) fn authorization_belongs_to_session(
        &self,
        session: WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        self.authorization()
            .is_none_or(|authorization| authorization.belongs_to_session(session))
    }

    pub(in crate::domain_computation) fn receipt(&self) -> WorthQueryApplicationDisclosureReceipt {
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

    pub(in crate::domain_computation) fn computation_matches(
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

    pub(in crate::domain_computation) fn readmission_matches(
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
