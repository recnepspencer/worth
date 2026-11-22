//! The installed residual filter: its admitted program and exact field reads.
use super::compilation::{admitted_aspect_key, admitted_field_key};
use std::sync::Arc;
use worth_foundational::facade::{AspectKey, CanonicalBasisEntry, FieldKey, ScalarAspectType};
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
}

impl WorthQueryInstalledWhereFilterParameter {
    pub fn operand(&self) -> &str {
        &self.operand
    }
    pub fn parameter(&self) -> &str {
        &self.parameter
    }
}

/// Residual narrowing over index-selected root candidates. The compiled
/// program is shared; the field list is the complete per-row read set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledWhereFilter {
    program: Arc<ApplicationQueryWhereFilterProgram>,
    fields: Vec<WorthQueryInstalledWhereFilterField>,
    parameters: Vec<WorthQueryInstalledWhereFilterParameter>,
}

impl WorthQueryInstalledWhereFilter {
    pub(super) fn compile(program: &ApplicationQueryWhereFilterProgram) -> Self {
        let mut fields = Vec::new();
        let mut parameters = Vec::new();
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
                Source::Parameter { parameter, .. } => {
                    parameters.push(WorthQueryInstalledWhereFilterParameter {
                        operand: operand.name().to_owned(),
                        parameter: parameter.clone(),
                    })
                }
            }
        }
        Self {
            program: Arc::new(program.clone()),
            fields,
            parameters,
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

    /// Graph and planning identity material; absent filters add nothing.
    pub(in crate::application_query) fn append_canonical(
        filter: Option<&Self>,
        entries: &mut Vec<CanonicalBasisEntry>,
        text: impl Fn(String, &str) -> CanonicalBasisEntry,
    ) {
        let Some(filter) = filter else {
            return;
        };
        for (index, field) in filter.fields.iter().enumerate() {
            let path = format!("where-filter.field[{index}]");
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
            let path = format!("where-filter.parameter[{index}]");
            entries.extend([
                text(format!("{path}.operand"), parameter.operand()),
                text(format!("{path}.parameter"), parameter.parameter()),
            ]);
        }
        entries.push(text(
            "where-filter.program".to_owned(),
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
