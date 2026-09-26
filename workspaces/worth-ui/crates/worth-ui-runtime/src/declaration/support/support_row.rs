use crate::declaration::{
    UiDeclaredHostCapabilityPosture, UiDeclaredMeasurementPolicyPosture,
    UiDeclaredPostureApplicability, UiDeclaredQueryBindingPosture, UiDeclaredServiceUsagePosture,
    UiDeclaredTouchMeaningPosture,
};

use super::{UiDeclarationSupportRowSchemaKind, UiDeclarationUnsupportedPosture};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiDeclarationSupportRow {
    schema_kind: UiDeclarationSupportRowSchemaKind,
    applicability: UiDeclaredPostureApplicability,
    admitted_fact: UiDeclarationSupportAdmittedFact,
    unsupported_posture: Option<UiDeclarationUnsupportedPosture>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum UiDeclarationSupportAdmittedFact {
    None,
    QueryBinding(UiDeclaredQueryBindingPosture),
    ServiceUsage(UiDeclaredServiceUsagePosture),
    TouchMeaning(UiDeclaredTouchMeaningPosture),
    MeasurementPolicy(UiDeclaredMeasurementPolicyPosture),
    HostCapability(UiDeclaredHostCapabilityPosture),
}

impl UiDeclarationSupportRow {
    fn new(
        schema_kind: UiDeclarationSupportRowSchemaKind,
        applicability: UiDeclaredPostureApplicability,
        admitted_fact: UiDeclarationSupportAdmittedFact,
        unsupported_posture: Option<UiDeclarationUnsupportedPosture>,
    ) -> Self {
        Self {
            schema_kind,
            applicability,
            admitted_fact,
            unsupported_posture,
        }
    }

    pub const fn schema_kind(&self) -> UiDeclarationSupportRowSchemaKind {
        self.schema_kind
    }

    pub const fn applicability(&self) -> UiDeclaredPostureApplicability {
        self.applicability
    }

    pub const fn unsupported_posture(&self) -> Option<UiDeclarationUnsupportedPosture> {
        self.unsupported_posture
    }

    pub fn declared_query_binding_posture(&self) -> Option<&UiDeclaredQueryBindingPosture> {
        match &self.admitted_fact {
            UiDeclarationSupportAdmittedFact::QueryBinding(posture) => Some(posture),
            UiDeclarationSupportAdmittedFact::None
            | UiDeclarationSupportAdmittedFact::ServiceUsage(_)
            | UiDeclarationSupportAdmittedFact::TouchMeaning(_)
            | UiDeclarationSupportAdmittedFact::MeasurementPolicy(_)
            | UiDeclarationSupportAdmittedFact::HostCapability(_) => None,
        }
    }

    pub fn declared_service_usage_posture(&self) -> Option<&UiDeclaredServiceUsagePosture> {
        match &self.admitted_fact {
            UiDeclarationSupportAdmittedFact::ServiceUsage(posture) => Some(posture),
            UiDeclarationSupportAdmittedFact::None
            | UiDeclarationSupportAdmittedFact::QueryBinding(_)
            | UiDeclarationSupportAdmittedFact::TouchMeaning(_)
            | UiDeclarationSupportAdmittedFact::MeasurementPolicy(_)
            | UiDeclarationSupportAdmittedFact::HostCapability(_) => None,
        }
    }

    pub fn declared_touch_meaning_posture(&self) -> Option<&UiDeclaredTouchMeaningPosture> {
        match &self.admitted_fact {
            UiDeclarationSupportAdmittedFact::TouchMeaning(posture) => Some(posture),
            UiDeclarationSupportAdmittedFact::None
            | UiDeclarationSupportAdmittedFact::QueryBinding(_)
            | UiDeclarationSupportAdmittedFact::ServiceUsage(_)
            | UiDeclarationSupportAdmittedFact::MeasurementPolicy(_)
            | UiDeclarationSupportAdmittedFact::HostCapability(_) => None,
        }
    }

    pub fn declared_measurement_policy_posture(
        &self,
    ) -> Option<&UiDeclaredMeasurementPolicyPosture> {
        match &self.admitted_fact {
            UiDeclarationSupportAdmittedFact::MeasurementPolicy(posture) => Some(posture),
            UiDeclarationSupportAdmittedFact::None
            | UiDeclarationSupportAdmittedFact::QueryBinding(_)
            | UiDeclarationSupportAdmittedFact::ServiceUsage(_)
            | UiDeclarationSupportAdmittedFact::TouchMeaning(_)
            | UiDeclarationSupportAdmittedFact::HostCapability(_) => None,
        }
    }

    pub fn declared_host_capability_posture(&self) -> Option<&UiDeclaredHostCapabilityPosture> {
        match &self.admitted_fact {
            UiDeclarationSupportAdmittedFact::HostCapability(posture) => Some(posture),
            UiDeclarationSupportAdmittedFact::None
            | UiDeclarationSupportAdmittedFact::QueryBinding(_)
            | UiDeclarationSupportAdmittedFact::ServiceUsage(_)
            | UiDeclarationSupportAdmittedFact::TouchMeaning(_)
            | UiDeclarationSupportAdmittedFact::MeasurementPolicy(_) => None,
        }
    }

    pub(crate) fn without_admitted_fact(
        schema_kind: UiDeclarationSupportRowSchemaKind,
        applicability: UiDeclaredPostureApplicability,
        unsupported_posture: Option<UiDeclarationUnsupportedPosture>,
    ) -> Self {
        Self::new(
            schema_kind,
            applicability,
            UiDeclarationSupportAdmittedFact::None,
            unsupported_posture,
        )
    }

    pub(crate) fn with_query_binding(
        schema_kind: UiDeclarationSupportRowSchemaKind,
        applicability: UiDeclaredPostureApplicability,
        posture: UiDeclaredQueryBindingPosture,
        unsupported_posture: Option<UiDeclarationUnsupportedPosture>,
    ) -> Self {
        Self::new(
            schema_kind,
            applicability,
            UiDeclarationSupportAdmittedFact::QueryBinding(posture),
            unsupported_posture,
        )
    }

    pub(crate) fn with_service_usage(
        schema_kind: UiDeclarationSupportRowSchemaKind,
        applicability: UiDeclaredPostureApplicability,
        posture: UiDeclaredServiceUsagePosture,
        unsupported_posture: Option<UiDeclarationUnsupportedPosture>,
    ) -> Self {
        Self::new(
            schema_kind,
            applicability,
            UiDeclarationSupportAdmittedFact::ServiceUsage(posture),
            unsupported_posture,
        )
    }

    pub(crate) fn with_touch_meaning(
        schema_kind: UiDeclarationSupportRowSchemaKind,
        applicability: UiDeclaredPostureApplicability,
        posture: UiDeclaredTouchMeaningPosture,
        unsupported_posture: Option<UiDeclarationUnsupportedPosture>,
    ) -> Self {
        Self::new(
            schema_kind,
            applicability,
            UiDeclarationSupportAdmittedFact::TouchMeaning(posture),
            unsupported_posture,
        )
    }

    pub(crate) fn with_measurement_policy(
        schema_kind: UiDeclarationSupportRowSchemaKind,
        applicability: UiDeclaredPostureApplicability,
        posture: UiDeclaredMeasurementPolicyPosture,
        unsupported_posture: Option<UiDeclarationUnsupportedPosture>,
    ) -> Self {
        Self::new(
            schema_kind,
            applicability,
            UiDeclarationSupportAdmittedFact::MeasurementPolicy(posture),
            unsupported_posture,
        )
    }

    pub(crate) fn with_host_capability(
        schema_kind: UiDeclarationSupportRowSchemaKind,
        applicability: UiDeclaredPostureApplicability,
        posture: UiDeclaredHostCapabilityPosture,
        unsupported_posture: Option<UiDeclarationUnsupportedPosture>,
    ) -> Self {
        Self::new(
            schema_kind,
            applicability,
            UiDeclarationSupportAdmittedFact::HostCapability(posture),
            unsupported_posture,
        )
    }
}
