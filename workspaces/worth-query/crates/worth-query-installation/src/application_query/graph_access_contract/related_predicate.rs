//! Installed related-record absence/existence predicates: traversed relation,
//! polarity (`Absent` / `Exists`), target-field equality predicates, and
//! optional Foundation target-entity `where_filter`.
use super::compilation::{admitted_aspect_key, admitted_field_key};
use super::WorthQueryInstalledWhereFilter;
use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectKey, AspectValue, CanonicalBasisEntry, FieldKey,
    ScalarAspectType,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryPredicateExpected, ApplicationQueryRelatedPolarity,
    ApplicationQueryRelatedPredicateMeaning, ApplicationQueryRelatedTargetPredicate,
    RelationDirection,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledRelatedTargetPredicate {
    entity: String,
    aspect: AspectKey,
    field: FieldKey,
    expected: ApplicationQueryPredicateExpected,
    scalar_family: ScalarAspectType,
    presence: ApplicationFieldPresence,
}

impl WorthQueryInstalledRelatedTargetPredicate {
    fn compile(predicate: &ApplicationQueryRelatedTargetPredicate) -> Self {
        let (entity, aspect, field) = predicate.field();
        Self {
            entity: entity.to_owned(),
            aspect: admitted_aspect_key(aspect),
            field: admitted_field_key(field),
            expected: predicate.expected().clone(),
            scalar_family: predicate.scalar_family(),
            presence: predicate.presence(),
        }
    }

    pub fn field(&self) -> (&str, &str, &str) {
        (&self.entity, self.aspect.as_str(), self.field.as_str())
    }

    pub const fn aspect_key(&self) -> &AspectKey {
        &self.aspect
    }

    pub const fn field_key(&self) -> &FieldKey {
        &self.field
    }

    pub const fn expected(&self) -> &ApplicationQueryPredicateExpected {
        &self.expected
    }

    pub fn parameter(&self) -> Option<&str> {
        match &self.expected {
            ApplicationQueryPredicateExpected::Parameter(parameter) => Some(parameter),
            ApplicationQueryPredicateExpected::Value(_) => None,
        }
    }

    pub fn literal_value(&self) -> Option<&AspectValue> {
        match &self.expected {
            ApplicationQueryPredicateExpected::Parameter(_) => None,
            ApplicationQueryPredicateExpected::Value(value) => Some(value),
        }
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub const fn presence(&self) -> ApplicationFieldPresence {
        self.presence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledRelatedPredicate {
    relation: String,
    from: String,
    to: String,
    direction: RelationDirection,
    polarity: ApplicationQueryRelatedPolarity,
    target_predicates: Vec<WorthQueryInstalledRelatedTargetPredicate>,
    target_where_filter: Option<WorthQueryInstalledWhereFilter>,
}

impl WorthQueryInstalledRelatedPredicate {
    pub(super) fn compile(meaning: &ApplicationQueryRelatedPredicateMeaning) -> Self {
        Self {
            relation: meaning.relation().to_owned(),
            from: meaning.from().to_owned(),
            to: meaning.to().to_owned(),
            direction: meaning.direction(),
            polarity: meaning.polarity(),
            target_predicates: meaning
                .target_predicates()
                .iter()
                .map(WorthQueryInstalledRelatedTargetPredicate::compile)
                .collect(),
            target_where_filter: meaning
                .target_where_filter()
                .map(WorthQueryInstalledWhereFilter::compile),
        }
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub const fn direction(&self) -> RelationDirection {
        self.direction
    }

    pub const fn polarity(&self) -> ApplicationQueryRelatedPolarity {
        self.polarity
    }

    pub fn target_entity(&self) -> &str {
        match self.direction {
            RelationDirection::Forward => &self.to,
            RelationDirection::Reverse => &self.from,
        }
    }

    pub fn target_predicates(&self) -> &[WorthQueryInstalledRelatedTargetPredicate] {
        &self.target_predicates
    }

    pub fn target_where_filter(&self) -> Option<&WorthQueryInstalledWhereFilter> {
        self.target_where_filter.as_ref()
    }

    /// Graph and planning identity material; empty slices emit zero entries.
    pub(in crate::application_query) fn append_canonical(
        predicates: &[Self],
        entries: &mut Vec<CanonicalBasisEntry>,
        text: impl Fn(String, &str) -> CanonicalBasisEntry + Copy,
    ) {
        for (index, predicate) in predicates.iter().enumerate() {
            let path = format!("related-predicate[{index}]");
            entries.extend([
                text(format!("{path}.relation"), predicate.relation()),
                text(format!("{path}.from"), predicate.from()),
                text(format!("{path}.to"), predicate.to()),
                text(
                    format!("{path}.direction"),
                    match predicate.direction() {
                        RelationDirection::Forward => "forward",
                        RelationDirection::Reverse => "reverse",
                    },
                ),
                text(
                    format!("{path}.polarity"),
                    predicate.polarity().canonical_name(),
                ),
            ]);
            for (target_index, target) in predicate.target_predicates.iter().enumerate() {
                let tpath = format!("{path}.target-predicate[{target_index}]");
                let (entity, aspect, field) = target.field();
                entries.extend([
                    text(format!("{tpath}.entity"), entity),
                    text(format!("{tpath}.aspect"), aspect),
                    text(format!("{tpath}.field"), field),
                    match target.expected() {
                        ApplicationQueryPredicateExpected::Parameter(parameter) => {
                            text(format!("{tpath}.parameter"), parameter)
                        }
                        ApplicationQueryPredicateExpected::Value(expected) => text(
                            format!("{tpath}.expected"),
                            prepare_aspect_value_identity_basis(expected).as_str(),
                        ),
                    },
                    text(
                        format!("{tpath}.scalar"),
                        target.scalar_family().canonical_name(),
                    ),
                    text(
                        format!("{tpath}.presence"),
                        (target.presence()).canonical_name(),
                    ),
                ]);
            }
            let filter_prefix = format!("{path}.target-where-filter");
            WorthQueryInstalledWhereFilter::append_canonical_at(
                &filter_prefix,
                predicate.target_where_filter.as_ref(),
                entries,
                text,
            );
        }
    }
}

impl super::WorthQueryInstalledGraphReadContract {
    /// Related-record absence/existence predicates narrowing selected roots.
    pub fn related_predicates(&self) -> &[WorthQueryInstalledRelatedPredicate] {
        &self.meaning().related_predicates
    }
}
