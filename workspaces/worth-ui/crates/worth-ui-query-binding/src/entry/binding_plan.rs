use std::collections::BTreeMap;

use crate::{
    UiApplicationScalarProjectionRegistration, UiCollectionProjectionRegistration,
    UiScalarProjectionRegistration, WorthUiInstalledQueryBindingReference,
    WorthUiInstalledQueryDomain, WorthUiInstalledQueryView, WorthUiQueryViewDefinition,
    WorthUiQueryViewIdentity,
};

use super::{
    WorthUiInstalledDownstreamQueryState, WorthUiQueryBindingRegistrationDenial,
    WorthUiQueryBindingRegistrationDenialKind, WorthUiRuntimeQueryBinding,
};

/// Stable app-owned binding plan. Query-free and installed Query posture are
/// explicit states; runtime authority never hides behind an optional field.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum WorthUiQueryBindingPlan {
    #[default]
    QueryFree,
    Installed(WorthUiInstalledQueryBindingPlan),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiInstalledQueryBindingPlan {
    installed_domain: Option<WorthUiInstalledQueryDomain>,
    references: BTreeMap<WorthUiQueryViewIdentity, WorthUiInstalledQueryBindingReference>,
    scalar_projections: BTreeMap<WorthUiQueryViewIdentity, UiScalarProjectionRegistration>,
    application_scalar_projections:
        BTreeMap<WorthUiQueryViewIdentity, UiApplicationScalarProjectionRegistration>,
    collection_projections: BTreeMap<WorthUiQueryViewIdentity, UiCollectionProjectionRegistration>,
    projection_slots: BTreeMap<WorthUiQueryViewIdentity, crate::UiProjectionInputSlot>,
}

impl WorthUiQueryBindingPlan {
    pub fn register_view(
        self,
        view: impl Into<WorthUiInstalledQueryView>,
    ) -> Result<Self, WorthUiQueryBindingRegistrationDenial> {
        let view = view.into();
        let (installed_domain, definition) = view.into_parts();
        let identity = definition.identity().clone();
        let reference =
            WorthUiInstalledQueryBindingReference::new(installed_domain.clone(), definition);
        match self {
            Self::QueryFree => {
                let mut references = BTreeMap::new();
                references.insert(identity, reference);
                Ok(Self::Installed(WorthUiInstalledQueryBindingPlan {
                    installed_domain: Some(installed_domain),
                    references,
                    scalar_projections: BTreeMap::new(),
                    application_scalar_projections: BTreeMap::new(),
                    collection_projections: BTreeMap::new(),
                    projection_slots: BTreeMap::new(),
                }))
            }
            Self::Installed(mut plan) => {
                if plan
                    .installed_domain
                    .as_ref()
                    .is_some_and(|current| !current.shares_authority_with(&installed_domain))
                {
                    return Err(WorthUiQueryBindingRegistrationDenial {
                        kind: WorthUiQueryBindingRegistrationDenialKind::ForeignInstalledDomain,
                        identity,
                    });
                }
                plan.installed_domain = Some(installed_domain.clone());
                if plan.references.contains_key(&identity) {
                    return Err(WorthUiQueryBindingRegistrationDenial {
                        kind: WorthUiQueryBindingRegistrationDenialKind::DuplicateViewIdentity,
                        identity,
                    });
                }
                plan.references.insert(identity, reference);
                Ok(Self::Installed(plan))
            }
        }
    }

    pub fn register_scalar_projection(
        self,
        registration: UiScalarProjectionRegistration,
    ) -> Result<Self, WorthUiQueryBindingRegistrationDenial> {
        let (installed_domain, identity) = registration.view().clone().into_parts();
        self.register_projection(
            installed_domain,
            identity,
            ProjectionRegistration::Scalar(registration),
        )
    }

    pub fn register_collection_projection(
        self,
        registration: UiCollectionProjectionRegistration,
    ) -> Result<Self, WorthUiQueryBindingRegistrationDenial> {
        let (installed_domain, identity) = registration.view().clone().into_parts();
        self.register_projection(
            installed_domain,
            identity,
            ProjectionRegistration::Collection(registration),
        )
    }

    pub fn register_application_scalar_projection(
        self,
        registration: UiApplicationScalarProjectionRegistration,
    ) -> Result<Self, WorthUiQueryBindingRegistrationDenial> {
        let identity = registration.identity().clone();
        let mut plan = match self {
            Self::QueryFree => WorthUiInstalledQueryBindingPlan {
                installed_domain: None,
                references: BTreeMap::new(),
                scalar_projections: BTreeMap::new(),
                application_scalar_projections: BTreeMap::new(),
                collection_projections: BTreeMap::new(),
                projection_slots: BTreeMap::new(),
            },
            Self::Installed(plan) => plan,
        };
        if plan.scalar_projections.contains_key(&identity)
            || plan.application_scalar_projections.contains_key(&identity)
            || plan.collection_projections.contains_key(&identity)
        {
            return Err(WorthUiQueryBindingRegistrationDenial {
                kind: WorthUiQueryBindingRegistrationDenialKind::DuplicateProjectionIdentity,
                identity,
            });
        }
        let slot = crate::UiProjectionInputSlot::from_index(plan.projection_slots.len())
            .ok_or_else(|| WorthUiQueryBindingRegistrationDenial {
                kind: WorthUiQueryBindingRegistrationDenialKind::ProjectionCapacityExceeded,
                identity: identity.clone(),
            })?;
        plan.projection_slots.insert(identity.clone(), slot);
        plan.application_scalar_projections
            .insert(identity, registration);
        Ok(Self::Installed(plan))
    }

    pub fn is_query_free(&self) -> bool {
        matches!(self, Self::QueryFree)
    }

    pub fn definitions(&self) -> Vec<&WorthUiQueryViewDefinition> {
        match self {
            Self::QueryFree => Vec::new(),
            Self::Installed(plan) => plan
                .references
                .values()
                .map(WorthUiInstalledQueryBindingReference::definition)
                .collect(),
        }
    }

    pub fn scalar_projection_registration(
        &self,
        identity: &WorthUiQueryViewIdentity,
    ) -> Option<&UiScalarProjectionRegistration> {
        match self {
            Self::QueryFree => None,
            Self::Installed(plan) => plan.scalar_projections.get(identity),
        }
    }

    pub fn collection_projection_registration(
        &self,
        identity: &WorthUiQueryViewIdentity,
    ) -> Option<&UiCollectionProjectionRegistration> {
        match self {
            Self::QueryFree => None,
            Self::Installed(plan) => plan.collection_projections.get(identity),
        }
    }

    pub fn application_scalar_projection_registration(
        &self,
        identity: &WorthUiQueryViewIdentity,
    ) -> Option<&UiApplicationScalarProjectionRegistration> {
        match self {
            Self::QueryFree => None,
            Self::Installed(plan) => plan.application_scalar_projections.get(identity),
        }
    }

    pub fn projection_identities(&self) -> Vec<WorthUiQueryViewIdentity> {
        match self {
            Self::QueryFree => Vec::new(),
            Self::Installed(plan) => plan
                .scalar_projections
                .keys()
                .chain(plan.application_scalar_projections.keys())
                .chain(plan.collection_projections.keys())
                .cloned()
                .collect(),
        }
    }

    pub fn projection_input_slot(
        &self,
        identity: &WorthUiQueryViewIdentity,
    ) -> Option<crate::UiProjectionInputSlot> {
        match self {
            Self::QueryFree => None,
            Self::Installed(plan) => plan.projection_slots.get(identity).copied(),
        }
    }

    pub fn projection_input_count(&self) -> usize {
        match self {
            Self::QueryFree => 0,
            Self::Installed(plan) => plan.projection_slots.len(),
        }
    }

    /// Resolve one compact lowering reference from this exact installed plan.
    pub fn resolve_definition(
        &self,
        identity: &WorthUiQueryViewIdentity,
        shape: crate::WorthUiQueryViewShape,
    ) -> Option<crate::WorthUiInstalledQueryBindingReference> {
        match self {
            Self::QueryFree => None,
            Self::Installed(plan) => plan.references.get(identity).and_then(|reference| {
                (reference.definition().shape() == shape).then(|| reference.clone())
            }),
        }
    }

    /// Verify that a compact reference still belongs to this exact installed
    /// plan rather than a semantically equal foreign Query runtime.
    pub fn admits_reference(
        &self,
        reference: &crate::WorthUiInstalledQueryBindingReference,
    ) -> bool {
        match self {
            Self::QueryFree => false,
            Self::Installed(plan) => {
                plan.references.get(reference.definition().identity()) == Some(reference)
            }
        }
    }

    /// Prepare UI-owned downstream fact retention. This does not create a
    /// Query execution root; operation attempts enter Query through the
    /// operating-world gateway.
    pub fn prepare_downstream_state(&self) -> WorthUiRuntimeQueryBinding {
        match self {
            Self::QueryFree => WorthUiRuntimeQueryBinding::QueryFree,
            Self::Installed(plan) => WorthUiRuntimeQueryBinding::Installed(Box::new(
                WorthUiInstalledDownstreamQueryState::new(
                    plan.references.clone(),
                    plan.scalar_projections.clone(),
                    plan.application_scalar_projections.clone(),
                    plan.collection_projections.clone(),
                ),
            )),
        }
    }

    fn register_projection(
        self,
        installed_domain: WorthUiInstalledQueryDomain,
        identity: WorthUiQueryViewIdentity,
        registration: ProjectionRegistration,
    ) -> Result<Self, WorthUiQueryBindingRegistrationDenial> {
        let mut plan = match self {
            Self::QueryFree => WorthUiInstalledQueryBindingPlan {
                installed_domain: Some(installed_domain),
                references: BTreeMap::new(),
                scalar_projections: BTreeMap::new(),
                application_scalar_projections: BTreeMap::new(),
                collection_projections: BTreeMap::new(),
                projection_slots: BTreeMap::new(),
            },
            Self::Installed(mut plan) => {
                if plan
                    .installed_domain
                    .as_ref()
                    .is_some_and(|current| !current.shares_authority_with(&installed_domain))
                {
                    return Err(WorthUiQueryBindingRegistrationDenial {
                        kind: WorthUiQueryBindingRegistrationDenialKind::ForeignInstalledDomain,
                        identity,
                    });
                }
                plan.installed_domain = Some(installed_domain);
                plan
            }
        };
        if plan.scalar_projections.contains_key(&identity)
            || plan.application_scalar_projections.contains_key(&identity)
            || plan.collection_projections.contains_key(&identity)
        {
            return Err(WorthUiQueryBindingRegistrationDenial {
                kind: WorthUiQueryBindingRegistrationDenialKind::DuplicateProjectionIdentity,
                identity,
            });
        }
        let slot = crate::UiProjectionInputSlot::from_index(plan.projection_slots.len())
            .ok_or_else(|| WorthUiQueryBindingRegistrationDenial {
                kind: WorthUiQueryBindingRegistrationDenialKind::ProjectionCapacityExceeded,
                identity: identity.clone(),
            })?;
        plan.projection_slots.insert(identity.clone(), slot);
        match registration {
            ProjectionRegistration::Scalar(registration) => {
                plan.scalar_projections.insert(identity, registration);
            }
            ProjectionRegistration::Collection(registration) => {
                plan.collection_projections.insert(identity, registration);
            }
        }
        Ok(Self::Installed(plan))
    }
}

enum ProjectionRegistration {
    Scalar(UiScalarProjectionRegistration),
    Collection(UiCollectionProjectionRegistration),
}

#[cfg(test)]
#[path = "binding_plan_tests.rs"]
mod tests;
