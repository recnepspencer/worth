//! The installed residual filter: its admitted program and exact field reads.
use super::compilation::{admitted_aspect_key, admitted_field_key};
use std::sync::Arc;
use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectKey, AspectValue, CanonicalBasisEntry, FieldKey,
    ScalarAspectType,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryWhereFilterProgram, ApplicationQueryWhereFilterSource as Source,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;

/// One root field the filter reads for every examined candidate, including
/// candidates it excludes. These are the filter's read footprint.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledWhereFilterField {
    operand: String,
    entity: String,
    aspect: AspectKey,
    field: FieldKey,
    scalar_family: ScalarAspectType,
    presence: ApplicationFieldPresence,
}

impl WorthQueryInstalledWhereFilterField {
    pub fn operand(&self) -> &str {
        &self.operand
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
    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }
    pub const fn presence(&self) -> ApplicationFieldPresence {
        self.presence
    }
}

/// An operand bound once per request from a declared query parameter.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledWhereFilterParameter {
    operand: String,
    parameter: String,
    optional: bool,
}

impl WorthQueryInstalledWhereFilterParameter {
    pub fn operand(&self) -> &str {
        &self.operand
    }
    pub fn parameter(&self) -> &str {
        &self.parameter
    }
    pub const fn is_optional(&self) -> bool {
        self.optional
    }
}

/// An operand bound once per query from an authored scalar constant.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledWhereFilterLiteral {
    operand: String,
    scalar_family: ScalarAspectType,
    value: AspectValue,
}

impl WorthQueryInstalledWhereFilterLiteral {
    pub fn operand(&self) -> &str {
        &self.operand
    }
    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }
    pub const fn value(&self) -> &AspectValue {
        &self.value
    }
}

/// Residual narrowing over index-selected root candidates. The compiled
/// program is shared; the field list is the complete per-row read set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledWhereFilter {
    program: Arc<ApplicationQueryWhereFilterProgram>,
    fields: Vec<WorthQueryInstalledWhereFilterField>,
    parameters: Vec<WorthQueryInstalledWhereFilterParameter>,
    literals: Vec<WorthQueryInstalledWhereFilterLiteral>,
}

impl WorthQueryInstalledWhereFilter {
    pub(super) fn compile(program: &ApplicationQueryWhereFilterProgram) -> Self {
        let mut fields = Vec::new();
        let mut parameters = Vec::new();
        let mut literals = Vec::new();
        for operand in program.operands() {
            match operand.source() {
                Source::Field {
                    entity,
                    aspect,
                    field,
                    scalar_family,
                    presence,
                } => fields.push(WorthQueryInstalledWhereFilterField {
                    operand: operand.name().to_owned(),
                    entity: entity.clone(),
                    aspect: admitted_aspect_key(aspect),
                    field: admitted_field_key(field),
                    scalar_family: *scalar_family,
                    presence: *presence,
                }),
                Source::Parameter { parameter, .. }
                | Source::OptionalParameter { parameter, .. } => {
                    parameters.push(WorthQueryInstalledWhereFilterParameter {
                        operand: operand.name().to_owned(),
                        parameter: parameter.clone(),
                        optional: matches!(operand.source(), Source::OptionalParameter { .. }),
                    })
                }
                Source::Literal {
                    scalar_family,
                    value,
                } => literals.push(WorthQueryInstalledWhereFilterLiteral {
                    operand: operand.name().to_owned(),
                    scalar_family: *scalar_family,
                    value: value.clone(),
                }),
            }
        }
        Self {
            program: Arc::new(program.clone()),
            fields,
            parameters,
            literals,
        }
    }

    pub fn program(&self) -> &ApplicationQueryWhereFilterProgram {
        &self.program
    }
    pub fn fields(&self) -> &[WorthQueryInstalledWhereFilterField] {
        &self.fields
    }
    pub fn parameters(&self) -> &[WorthQueryInstalledWhereFilterParameter] {
        &self.parameters
    }
    pub fn literals(&self) -> &[WorthQueryInstalledWhereFilterLiteral] {
        &self.literals
    }

    /// Graph and planning identity material; absent filters add nothing.
    pub(in crate::application_query) fn append_canonical(
        filter: Option<&Self>,
        entries: &mut Vec<CanonicalBasisEntry>,
        text: impl Fn(String, &str) -> CanonicalBasisEntry,
    ) {
        Self::append_canonical_at("where-filter", filter, entries, text);
    }

    pub(in crate::application_query) fn append_canonical_at(
        prefix: &str,
        filter: Option<&Self>,
        entries: &mut Vec<CanonicalBasisEntry>,
        text: impl Fn(String, &str) -> CanonicalBasisEntry,
    ) {
        let Some(filter) = filter else {
            return;
        };
        for (index, field) in filter.fields.iter().enumerate() {
            let path = format!("{prefix}.field[{index}]");
            let (entity, aspect, name) = field.field();
            entries.extend([
                text(format!("{path}.operand"), field.operand()),
                text(format!("{path}.entity"), entity),
                text(format!("{path}.aspect"), aspect),
                text(format!("{path}.field"), name),
                text(
                    format!("{path}.scalar"),
                    field.scalar_family.canonical_name(),
                ),
                text(
                    format!("{path}.presence"),
                    match field.presence {
                        ApplicationFieldPresence::Required => "required",
                        ApplicationFieldPresence::Optional => "optional",
                    },
                ),
            ]);
        }
        for (index, parameter) in filter.parameters.iter().enumerate() {
            let path = format!("{prefix}.parameter[{index}]");
            entries.extend([
                text(format!("{path}.operand"), parameter.operand()),
                text(format!("{path}.parameter"), parameter.parameter()),
            ]);
            if parameter.is_optional() {
                entries.push(text(format!("{path}.presence"), "optional"));
            }
        }
        for (index, literal) in filter.literals.iter().enumerate() {
            let path = format!("{prefix}.literal[{index}]");
            entries.extend([
                text(format!("{path}.operand"), literal.operand()),
                text(
                    format!("{path}.scalar"),
                    literal.scalar_family().canonical_name(),
                ),
                text(
                    format!("{path}.value"),
                    prepare_aspect_value_identity_basis(literal.value()).as_str(),
                ),
            ]);
        }
        entries.push(text(
            format!("{prefix}.program"),
            filter.program.semantic_material(),
        ));
    }
}

impl super::WorthQueryInstalledGraphReadContract {
    /// The residual filter narrowing index-selected candidates, if declared.
    pub fn where_filter(&self) -> Option<&WorthQueryInstalledWhereFilter> {
        self.meaning().where_filter.as_ref()
    }
}
