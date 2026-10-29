use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{
    AspectMask, AspectValue, DiagnosticMask, FieldKey, ProjectionMask,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryDisclosurePosture, ApplicationQueryDisclosureRule,
    ApplicationQueryDisclosureSelector, ApplicationQueryInfluenceContract,
    ApplicationQueryResultSlotKey,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

use super::influence_validation::{validate_influence, GovernedInternalFieldRules};
pub(super) mod admission;
mod compilation;
mod field_masks;
mod result_rules;
pub(in crate::domain_computation) use compilation::compile_disclosure_contract;
pub(in crate::domain_computation) use field_masks::admit_field_mask_categories;
pub(super) use result_rules::selector_parts;
mod diagnostic;
mod history;
mod inspection;
pub(super) use diagnostic::diagnostic_shape_supported;
pub(in crate::domain_computation) use diagnostic::{
    compile_diagnostic_disclosure_contract, DiagnosticContractDenial,
};
mod resource;
pub(in crate::domain_computation) use history::compile_history_disclosure_contract;
pub(in crate::domain_computation) use inspection::compile_inspection_disclosure_contract;
pub(in crate::domain_computation) use resource::compile_resource_disclosure_contract;

#[derive(Clone, Debug)]
pub(super) struct WorthQueryAdmittedApplicationDisclosureField {
    entity: String,
    aspect: String,
    field: String,
    projection_mask: AspectMask<ProjectionMask>,
    diagnostic_mask: AspectMask<DiagnosticMask>,
    bound_scalar: bool,
}

impl WorthQueryAdmittedApplicationDisclosureField {
    pub(super) fn matches(&self, field: (&str, &str, &str)) -> bool {
        self.entity == field.0 && self.aspect == field.1 && self.field == field.2
    }

    pub(super) const fn projection_mask(&self) -> &AspectMask<ProjectionMask> {
        &self.projection_mask
    }

    pub(super) const fn diagnostic_mask(&self) -> &AspectMask<DiagnosticMask> {
        &self.diagnostic_mask
    }

    pub(super) fn is_bound_scalar(&self, field: &str) -> bool {
        self.bound_scalar && self.field == field
    }

    pub(super) fn admits_projection_field(&self, field: &FieldKey) -> bool {
        self.field == field.as_str()
            && ((self.is_bound_scalar(field.as_str()) && self.projection_mask.is_whole_aspect())
                || self
                    .projection_mask
                    .paths()
                    .iter()
                    .any(|path| path.fields() == std::slice::from_ref(field)))
    }

    pub(super) fn admits_diagnostic_field(&self, field: &str) -> bool {
        self.field == field
            && ((self.is_bound_scalar(field) && self.diagnostic_mask.is_whole_aspect())
                || self
                    .diagnostic_mask
                    .paths()
                    .iter()
                    .any(|path| matches!(path.fields(), [key] if key.as_str() == field)))
    }
}

#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct WorthQueryAdmittedApplicationDisclosureRule {
    disclosure_value: AspectValue,
    influence: ApplicationQueryInfluenceContract,
    field: Option<WorthQueryAdmittedApplicationDisclosureField>,
}

impl WorthQueryAdmittedApplicationDisclosureRule {
    pub(super) const fn disclosure_value(&self) -> &AspectValue {
        &self.disclosure_value
    }

    pub(super) const fn influence(&self) -> &ApplicationQueryInfluenceContract {
        &self.influence
    }

    pub(super) const fn field(&self) -> Option<&WorthQueryAdmittedApplicationDisclosureField> {
        self.field.as_ref()
    }
}

pub(super) type AdmittedDisclosureRule = Arc<WorthQueryAdmittedApplicationDisclosureRule>;

#[derive(Clone, Debug)]
pub(in crate::domain_computation) enum WorthQueryAdmittedApplicationDisclosureContract {
    Public,
    Governed {
        classification: String,
        capability_name: String,
        capability_type: String,
        result_rules: BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
        internal_rules: Vec<AdmittedDisclosureRule>,
        internal_field_rules: GovernedInternalFieldRules,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryApplicationDisclosureContractDenial {
    subject: String,
}

impl WorthQueryApplicationDisclosureContractDenial {
    pub(in crate::domain_computation) fn subject(&self) -> &str {
        &self.subject
    }
}

pub(super) fn denial(subject: impl Into<String>) -> WorthQueryApplicationDisclosureContractDenial {
    WorthQueryApplicationDisclosureContractDenial {
        subject: subject.into(),
    }
}
