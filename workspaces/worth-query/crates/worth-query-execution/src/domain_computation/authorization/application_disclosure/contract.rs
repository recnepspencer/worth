use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectMask, AspectValue, DiagnosticMask, ProjectionMask};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryDisclosurePosture, ApplicationQueryDisclosureRule,
    ApplicationQueryDisclosureSelector, ApplicationQueryInfluenceContract,
    ApplicationQueryResultSlotKey,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

use super::influence_validation::{validate_influence, GovernedInternalFieldRules};

#[derive(Clone, Debug)]
pub(super) struct WorthQueryAdmittedApplicationDisclosureField {
    entity: String,
    aspect: String,
    field: String,
    projection_mask: AspectMask<ProjectionMask>,
    _diagnostic_mask: AspectMask<DiagnosticMask>,
}

impl WorthQueryAdmittedApplicationDisclosureField {
    pub(super) fn matches(&self, field: (&str, &str, &str)) -> bool {
        self.entity == field.0 && self.aspect == field.1 && self.field == field.2
    }

    pub(super) const fn projection_mask(&self) -> &AspectMask<ProjectionMask> {
        &self.projection_mask
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

pub(in crate::domain_computation) fn compile_disclosure_contract<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<
    WorthQueryAdmittedApplicationDisclosureContract,
    WorthQueryApplicationDisclosureContractDenial,
> {
    match query.disclosure().posture() {
        ApplicationQueryDisclosurePosture::Public if query.disclosure().rules().is_empty() => {
            Ok(WorthQueryAdmittedApplicationDisclosureContract::Public)
        }
        ApplicationQueryDisclosurePosture::Public
        | ApplicationQueryDisclosurePosture::InstalledPolicyRequired => Err(denial(query.name())),
        ApplicationQueryDisclosurePosture::Governed => compile_governed_contract(query, layout),
    }
}

fn compile_governed_contract<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<
    WorthQueryAdmittedApplicationDisclosureContract,
    WorthQueryApplicationDisclosureContractDenial,
> {
    let mut result_rules = BTreeMap::new();
    let mut internal_rules = Vec::new();
    let mut internal_field_rules = BTreeMap::new();
    for rule in query.disclosure().rules() {
        admit_rule(
            query,
            layout,
            rule,
            &mut result_rules,
            &mut internal_rules,
            &mut internal_field_rules,
        )?;
    }
    require_complete_result_shape(query, &result_rules)?;
    validate_influence(query, &internal_field_rules, &result_rules)?;
    Ok(WorthQueryAdmittedApplicationDisclosureContract::Governed {
        classification: query.disclosure().classification().to_string(),
        capability_name: query
            .disclosure()
            .capability_name()
            .ok_or_else(|| denial(query.name()))?
            .to_string(),
        capability_type: query
            .disclosure()
            .capability_type()
            .ok_or_else(|| denial(query.name()))?
            .to_string(),
        result_rules,
        internal_rules,
        internal_field_rules,
    })
}

fn admit_rule<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
    rule: &ApplicationQueryDisclosureRule,
    result_rules: &mut BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
    internal_rules: &mut Vec<AdmittedDisclosureRule>,
    internal_field_rules: &mut GovernedInternalFieldRules,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    let field = rule
        .selector()
        .field_contract()
        .map(|field| admit_field_masks(layout, rule.selector(), field))
        .transpose()?;
    let installed = Arc::new(WorthQueryAdmittedApplicationDisclosureRule {
        disclosure_value: rule.disclosure_value().clone(),
        influence: rule.influence().clone(),
        field,
    });
    match rule.selector() {
        ApplicationQueryDisclosureSelector::InternalField { .. } => {
            let field = installed
                .field()
                .ok_or_else(|| denial(rule.selector().slot_type()))?;
            internal_field_rules
                .entry((
                    field.entity.clone(),
                    field.aspect.clone(),
                    field.field.clone(),
                ))
                .or_default()
                .push(Arc::clone(&installed));
            internal_rules.push(installed);
        }
        ApplicationQueryDisclosureSelector::Field { .. } => {
            admit_result_rule(query, rule.selector(), installed, result_rules, true)?;
        }
        ApplicationQueryDisclosureSelector::Relation { .. } => {
            admit_result_rule(query, rule.selector(), installed, result_rules, false)?;
        }
        ApplicationQueryDisclosureSelector::Orbit(orbit) => {
            let slot = orbit.slot_key();
            if !query
                .read_graph()
                .orbit()
                .is_some_and(|expected| expected.slot_key() == slot)
                || result_rules.insert(slot, installed).is_some()
            {
                return Err(denial(orbit.slot_type()));
            }
        }
    }
    Ok(())
}

fn admit_result_rule<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    selector: &ApplicationQueryDisclosureSelector,
    rule: AdmittedDisclosureRule,
    result_rules: &mut BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
    field: bool,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    let slot = selector
        .result_slot_key()
        .ok_or_else(|| denial(selector.slot_type()))?;
    let exact = if field {
        query
            .read_graph()
            .projections()
            .iter()
            .any(|projection| projection.slot_key_identity().as_ref() == &slot)
    } else {
        query
            .read_graph()
            .relations()
            .iter()
            .any(|relation| relation.slot_key_identity().as_ref() == &slot)
    };
    if exact && result_rules.insert(slot, rule).is_none() {
        Ok(())
    } else {
        Err(denial(selector.slot_type()))
    }
}

fn admit_field_masks(
    layout: &WorthQueryPrimaryGraphLayout,
    selector: &ApplicationQueryDisclosureSelector,
    field: (&str, &str, &str),
) -> Result<
    WorthQueryAdmittedApplicationDisclosureField,
    WorthQueryApplicationDisclosureContractDenial,
> {
    let (projection_mask, diagnostic_mask) = admit_field_mask_categories(layout, selector, field)?;
    Ok(WorthQueryAdmittedApplicationDisclosureField {
        entity: field.0.to_string(),
        aspect: field.1.to_string(),
        field: field.2.to_string(),
        projection_mask,
        _diagnostic_mask: diagnostic_mask,
    })
}

pub(in crate::domain_computation) fn admit_field_mask_categories(
    layout: &WorthQueryPrimaryGraphLayout,
    selector: &ApplicationQueryDisclosureSelector,
    field: (&str, &str, &str),
) -> Result<
    (AspectMask<ProjectionMask>, AspectMask<DiagnosticMask>),
    WorthQueryApplicationDisclosureContractDenial,
> {
    let aspect_key =
        worth_foundational::facade::AspectKey::new(field.1).ok_or_else(|| denial(field.1))?;
    let contract = layout
        .aspect_contract(field.0, &aspect_key)
        .ok_or_else(|| denial(field.1))?;
    let projection_mask = selector
        .projection_mask()
        .ok_or_else(|| denial(field.2))?
        .clone();
    let diagnostic_mask = selector
        .diagnostic_mask()
        .ok_or_else(|| denial(field.2))?
        .clone();
    contract
        .admits_projection_mask(&projection_mask)
        .map_err(|_| denial(field.1))?;
    contract
        .admits_diagnostic_mask(&diagnostic_mask)
        .map_err(|_| denial(field.1))?;
    Ok((projection_mask, diagnostic_mask))
}

fn require_complete_result_shape<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    rules: &BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    let orbit = query.read_graph().orbit();
    let expected = query.read_graph().projections().len()
        + query.read_graph().relations().len()
        + usize::from(orbit.is_some());
    let exact = query
        .read_graph()
        .projections()
        .iter()
        .map(|projection| projection.slot_key_identity())
        .chain(
            query
                .read_graph()
                .relations()
                .iter()
                .map(|relation| relation.slot_key_identity()),
        )
        .all(|slot| rules.contains_key(slot.as_ref()));
    let orbit_exact = orbit.is_none_or(|orbit| rules.contains_key(&orbit.slot_key()));
    (expected == rules.len() && exact && orbit_exact)
        .then_some(())
        .ok_or_else(|| denial(query.name()))
}

pub(super) fn denial(subject: impl Into<String>) -> WorthQueryApplicationDisclosureContractDenial {
    WorthQueryApplicationDisclosureContractDenial {
        subject: subject.into(),
    }
}
