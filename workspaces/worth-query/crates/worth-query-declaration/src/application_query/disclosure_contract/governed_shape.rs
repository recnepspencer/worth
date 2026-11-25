//! Governs a declared result shape as a whole: every field and relation slot,
//! nested relations included, under one disclosure value and influence.
use worth_foundational::facade::{AspectMask, AspectValue, CanonicalFieldPath, FieldKey};

use super::{
    ApplicationQueryDisclosureContract, ApplicationQueryDisclosureRule,
    ApplicationQueryDisclosureSelector, ApplicationQueryInfluenceContract,
};
use crate::application_query::ApplicationQueryResultShape;

impl ApplicationQueryDisclosureContract {
    /// Adds one disclosure rule per field and relation slot of `shape`.
    ///
    /// Refuses (adding nothing) when a field name is not a Foundational key,
    /// which a shape built from typed field selections never carries.
    pub(crate) fn disclose_shape_by(
        &mut self,
        shape: &ApplicationQueryResultShape,
        disclosure_value: &AspectValue,
        influence: &ApplicationQueryInfluenceContract,
    ) -> Result<(), InvalidShapeFieldKey> {
        let mut rules = Vec::new();
        collect_shape_rules(shape, disclosure_value, influence, &mut rules)?;
        self.rules.extend(rules);
        self.rules.sort();
        Ok(())
    }
}

/// A result shape field whose name is not a valid Foundational key.
#[derive(Debug)]
pub(crate) struct InvalidShapeFieldKey;

fn collect_shape_rules(
    shape: &ApplicationQueryResultShape,
    disclosure_value: &AspectValue,
    influence: &ApplicationQueryInfluenceContract,
    rules: &mut Vec<ApplicationQueryDisclosureRule>,
) -> Result<(), InvalidShapeFieldKey> {
    for field in shape.fields() {
        let key = FieldKey::new(field.field()).ok_or(InvalidShapeFieldKey)?;
        rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Field {
                query_type: field.query_identity(),
                slot_type: field.slot_identity(),
                entity: field.entity().to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
                output_name: field.output_name().to_owned(),
                scalar_family: field.scalar_family(),
                value_type: field.value_identity(),
                presence: field.presence(),
                projection_mask: AspectMask::new([CanonicalFieldPath::single(key.clone())]),
                diagnostic_mask: AspectMask::new([CanonicalFieldPath::single(key)]),
            },
            disclosure_value: disclosure_value.clone(),
            influence: influence.clone(),
        });
    }
    for relation in shape.relations() {
        rules.push(ApplicationQueryDisclosureRule {
            selector: ApplicationQueryDisclosureSelector::Relation {
                query_type: relation.query_identity(),
                slot_type: relation.slot_identity(),
                relation: relation.relation().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
                direction: relation.direction(),
                cardinality: relation.cardinality(),
                output_name: relation.output_name().to_owned(),
            },
            disclosure_value: disclosure_value.clone(),
            influence: influence.clone(),
        });
        collect_shape_rules(relation.nested_shape(), disclosure_value, influence, rules)?;
    }
    Ok(())
}
