use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{
    AspectMask, AspectValue, DiagnosticMask, FieldKey, ProjectionMask,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryDisclosurePosture, ApplicationQueryDisclosureRule,
    ApplicationQueryDisclosureSelector, ApplicationQueryInfluenceContract,
    ApplicationQueryObservableInfluence, ApplicationQueryResultSlotKey, RelationDirection,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

use super::influence_validation::{
    validate_influence, GovernedInternalFieldRules, GovernedInternalRelationRules,
};
pub mod admission;
pub mod compilation;
pub mod field_masks;
pub mod funded;
pub mod result_rules;
pub use compilation::compile_disclosure_contract;
pub use funded::{
    compile_funded_disclosure_contract, FundedDisclosureCompilationDenial, FundedDisclosureContract,
};
pub use result_rules::selector_parts;
pub mod diagnostic;
pub mod history;
pub mod inspection;
pub use diagnostic::compile_diagnostic_disclosure_contract;
pub use diagnostic::diagnostic_shape_supported;
pub mod resource;
pub use history::compile_history_disclosure_contract;
pub use inspection::compile_inspection_disclosure_contract;
pub use resource::compile_resource_disclosure_contract;

#[derive(Clone, Debug)]
pub struct WorthQueryAdmittedApplicationDisclosureField {
    entity: String,
    aspect: String,
    field: String,
    projection_mask: AspectMask<ProjectionMask>,
    diagnostic_mask: AspectMask<DiagnosticMask>,
    bound_scalar: bool,
}

impl WorthQueryAdmittedApplicationDisclosureField {
    pub fn matches(&self, field: (&str, &str, &str)) -> bool {
        self.entity == field.0 && self.aspect == field.1 && self.field == field.2
    }

    pub const fn projection_mask(&self) -> &AspectMask<ProjectionMask> {
        &self.projection_mask
    }

    pub const fn diagnostic_mask(&self) -> &AspectMask<DiagnosticMask> {
        &self.diagnostic_mask
    }

    pub fn is_bound_scalar(&self, field: &str) -> bool {
        self.bound_scalar && self.field == field
    }

    pub fn admits_projection_field(&self, field: &FieldKey) -> bool {
        self.field == field.as_str()
            && ((self.is_bound_scalar(field.as_str()) && self.projection_mask.is_whole_aspect())
                || self
                    .projection_mask
                    .paths()
                    .iter()
                    .any(|path| path.fields() == std::slice::from_ref(field)))
    }

    pub fn admits_diagnostic_field(&self, field: &str) -> bool {
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
pub struct WorthQueryAdmittedApplicationDisclosureRule {
    disclosure_value: AspectValue,
    influence: ApplicationQueryInfluenceContract,
    field: Option<WorthQueryAdmittedApplicationDisclosureField>,
}

impl WorthQueryAdmittedApplicationDisclosureRule {
    pub const fn disclosure_value(&self) -> &AspectValue {
        &self.disclosure_value
    }

    pub const fn influence(&self) -> &ApplicationQueryInfluenceContract {
        &self.influence
    }

    pub const fn field(&self) -> Option<&WorthQueryAdmittedApplicationDisclosureField> {
        self.field.as_ref()
    }
}

pub type AdmittedDisclosureRule = Arc<WorthQueryAdmittedApplicationDisclosureRule>;

pub(crate) const fn influence_bit(surface: ApplicationQueryObservableInfluence) -> u16 {
    match surface {
        ApplicationQueryObservableInfluence::RowPresence => 1 << 0,
        ApplicationQueryObservableInfluence::Ordering => 1 << 1,
        ApplicationQueryObservableInfluence::Pagination => 1 << 2,
        ApplicationQueryObservableInfluence::Count => 1 << 3,
        ApplicationQueryObservableInfluence::Aggregate => 1 << 4,
        ApplicationQueryObservableInfluence::Explanation => 1 << 5,
        ApplicationQueryObservableInfluence::HistoricalMembership => 1 << 6,
        ApplicationQueryObservableInfluence::Preview => 1 << 7,
        ApplicationQueryObservableInfluence::LiveMembership => 1 << 8,
    }
}

/// Pre-intersected rule group for one internal selector so runtime admission
/// is strictly one O(log K) binary search plus one O(1) bitmask check.
#[derive(Clone, Debug)]
pub struct GovernedInternalRuleGroup {
    rules: Vec<AdmittedDisclosureRule>,
    permitted_mask: u16,
}

impl GovernedInternalRuleGroup {
    pub(crate) fn new(rules: Vec<AdmittedDisclosureRule>, permitted_mask: u16) -> Self {
        Self {
            rules,
            permitted_mask,
        }
    }

    pub fn rules(&self) -> &[AdmittedDisclosureRule] {
        &self.rules
    }

    pub const fn permits(&self, surface: ApplicationQueryObservableInfluence) -> bool {
        (self.permitted_mask & influence_bit(surface)) != 0
    }

    pub fn field(&self) -> Option<&WorthQueryAdmittedApplicationDisclosureField> {
        self.rules.first()?.field()
    }
}

pub type SortedGovernedInternalFieldRules =
    Vec<((String, String, String), GovernedInternalRuleGroup)>;
pub type SortedGovernedInternalRelationRules = Vec<(
    (String, String, String, RelationDirection),
    GovernedInternalRuleGroup,
)>;

#[derive(Clone, Debug)]
pub enum WorthQueryAdmittedApplicationDisclosureContract {
    Public,
    Governed {
        classification: String,
        capability_name: String,
        capability_type: String,
        result_rules: BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
        internal_rules: Vec<AdmittedDisclosureRule>,
        internal_field_rules: SortedGovernedInternalFieldRules,
        internal_relation_rules: SortedGovernedInternalRelationRules,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationDisclosureContractDenial {
    subject: String,
}

impl WorthQueryApplicationDisclosureContractDenial {
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

pub fn denial(subject: impl Into<String>) -> WorthQueryApplicationDisclosureContractDenial {
    WorthQueryApplicationDisclosureContractDenial {
        subject: subject.into(),
    }
}

pub use diagnostic::DiagnosticContractDenial;
