use super::error::{EvidenceReportError, EvidenceReportErrorKind};
use super::field::{EvidenceReportField, EvidenceReportFieldValue};
use super::identity::derive_evidence_report_identities;
use super::participation::EvidenceReportFieldParticipation;
use super::report::EvidenceReport;
use super::scope::EvidenceReportScope;
use crate::WorthQueryEvidenceIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceReportDeclaration {
    scope: EvidenceReportScope,
    report_name: String,
    fields: Vec<EvidenceReportField>,
}

impl EvidenceReportDeclaration {
    pub fn new(
        scope: EvidenceReportScope,
        report_name: impl Into<String>,
    ) -> Result<Self, EvidenceReportError> {
        let report_name = report_name.into();
        if report_name.is_empty() {
            return Err(EvidenceReportError::new(
                EvidenceReportErrorKind::EmptyReportName,
                "evidence report name must not be empty",
            ));
        }
        Ok(Self {
            scope,
            report_name,
            fields: Vec::new(),
        })
    }

    pub fn shape_participating(
        self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, EvidenceReportError> {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::Shape(value.into()),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn value_participating(
        self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, EvidenceReportError> {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::Value(value.into()),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn bool_participating(
        self,
        name: impl Into<String>,
        value: bool,
    ) -> Result<Self, EvidenceReportError> {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::Bool(value),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn usize_participating(
        self,
        name: impl Into<String>,
        value: usize,
    ) -> Result<Self, EvidenceReportError> {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::Usize(value),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn identity_participating(
        self,
        name: impl Into<String>,
        value: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, EvidenceReportError> {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::EvidenceIdentity(value.clone()),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn value_sequence_participating<I, S>(
        self,
        name: impl Into<String>,
        values: I,
    ) -> Result<Self, EvidenceReportError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::ValueSequence(values.into_iter().map(Into::into).collect()),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn identity_sequence_participating<'a, I>(
        self,
        name: impl Into<String>,
        values: I,
    ) -> Result<Self, EvidenceReportError>
    where
        I: IntoIterator<Item = &'a WorthQueryEvidenceIdentity>,
    {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::EvidenceIdentitySequence(
                values.into_iter().cloned().collect(),
            ),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn optional_value_participating(
        self,
        name: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> Result<Self, EvidenceReportError> {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::OptionalValue(value.map(Into::into)),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn optional_identity_participating(
        self,
        name: impl Into<String>,
        value: Option<&WorthQueryEvidenceIdentity>,
    ) -> Result<Self, EvidenceReportError> {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::OptionalEvidenceIdentity(value.cloned()),
            EvidenceReportFieldParticipation::Participating,
        )
    }

    pub fn diagnostic_value_nonparticipating(
        self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, EvidenceReportError> {
        self.append_declared_field(
            name,
            EvidenceReportFieldValue::Value(value.into()),
            EvidenceReportFieldParticipation::DiagnosticNonParticipating,
        )
    }

    pub fn seal(self) -> Result<EvidenceReport, EvidenceReportError> {
        if !self
            .fields
            .iter()
            .any(|field| field.participation().participates_in_report_identity())
        {
            return Err(EvidenceReportError::new(
                EvidenceReportErrorKind::MissingParticipatingField,
                "evidence report must have at least one participating field",
            ));
        }

        let identities =
            derive_evidence_report_identities(&self.scope, &self.report_name, &self.fields)?;
        EvidenceReport::sealed(self.scope, self.report_name, self.fields, identities)
    }

    fn append_declared_field(
        mut self,
        name: impl Into<String>,
        value: EvidenceReportFieldValue,
        participation: EvidenceReportFieldParticipation,
    ) -> Result<Self, EvidenceReportError> {
        let field = EvidenceReportField::new(name, value, participation)?;
        if self
            .fields
            .iter()
            .any(|existing| existing.name() == field.name())
        {
            return Err(EvidenceReportError::new(
                EvidenceReportErrorKind::DuplicateFieldName,
                format!("duplicate evidence report field `{}`", field.name()),
            ));
        }
        self.fields.push(field);
        Ok(self)
    }
}
