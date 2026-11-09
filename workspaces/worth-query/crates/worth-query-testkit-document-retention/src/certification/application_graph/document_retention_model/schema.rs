//! The authored meaning both document-retention programs are written against.
//!
//! One schema installs one document entity carrying a retention period in
//! days, one operation that sets it, one ordinary read, and BOTH
//! document-retention rule contracts. Which of the two actually decides a
//! candidate is not a property of this schema: it is decided by the program
//! the occurrence is running.

#[path = "schema/records.rs"]
mod storage_model;
pub use storage_model::*;
pub use worth_query_host::facade::{declaration, primary_graph};
pub use worth_query_host::facade::{
    worth_query_application_contribution, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_portable_type, worth_query_structured_value_binding,
};

#[path = "schema/document_retention_invariants.rs"]
pub mod document_retention_invariants;
#[path = "schema/document_retention_query.rs"]
pub mod document_retention_query;

pub use document_retention_invariants::{
    DocumentRetentionV1, DocumentRetentionV2, DocumentRetentionV3,
};
pub use document_retention_query::{
    document_retention_condition_query_definition, document_retention_query_definition,
    DocumentQueryParameters, DocumentQueryParametersBinding, DocumentRetentionConditionBinding,
    DocumentRetentionConditionQuery, DocumentRetentionQuery, DocumentRetentionRow,
    DocumentRetentionRowBinding,
};

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy, Debug)]
pub struct DocumentRetentionSchema;
impl Fragment for DocumentRetentionSchema {
    const NAME: &'static str = "document_retention";
    type Bindings = NoBindings;
    type Members = (DocumentRetentionContribution,);
}
impl ApplicationSchema for DocumentRetentionSchema {
    const OWNER: &'static str = "document_retention_courtroom";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl DocumentRetentionSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for DocumentRetentionSchema {
    type Contributions = <Self as Fragment>::Members;
}
pub struct DocumentRetentionContribution;
impl ApplicationSchemaContribution<DocumentRetentionSchema> for DocumentRetentionContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new(
            "document_retention_courtroom.application_graph.v1",
        );
    type Members = (
        RetentionModel,
        DocumentPrincipalBinding,
        super::workflow::declaration::WorkflowGrants,
        super::document_history::control_roles::ControlGrants,
        RetentionPrograms,
        super::correctable_retention::RestoreReviewedRetention,
    );
}
impl DocumentRetentionContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<DocumentRetentionSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct RetentionPrograms;
impl Registration<DocumentRetentionSchema> for RetentionPrograms {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = super::document_history::declare(schema);
            let schema = super::model_lifecycle::declare(schema);
            let schema = schema
                .operation(
                    SetRetention::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_projection_work_budget(SetRetention::reference(), 8)
                .operation_read_field(
                    SetRetention::reference(),
                    DocumentIdentityField::at("DocumentIdentityField"),
                )
                .operation_read_field(
                    SetRetention::reference(),
                    DocumentRetentionField::at("DocumentRetentionField"),
                )
                .operation_expected_fact(
                    SetRetention::reference(),
                    DocumentRetentionField::at("DocumentRetentionField"),
                )
                .operation_write(
                    SetRetention::reference(),
                    DocumentRetentionField::at("DocumentRetentionField"),
                )
                .invariant(document_retention_invariants::first_definition())
                .invariant(document_retention_invariants::second_definition())
                .application_query(document_retention_query_definition())
                .application_query(document_retention_condition_query_definition())
                .application_query(
                    super::retention_days::document_retention_days_query_definition(),
                );
            super::record_history::declare(super::correctable_retention::declare(
                super::actor_generation::declare(super::program_adoption::declare(
                    super::workflow::declare(super::assessment_output::declare(
                        super::retention_days::declare(super::retention_entry::declare(schema)),
                    )),
                )),
            ))
        })
    }
}

pub struct DocumentPrincipalBinding;
impl DocumentPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "DocumentPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}
impl Registration<DocumentRetentionSchema> for DocumentPrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}

/// The exact retention one request asks a named document to carry.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct SetRetentionInput {
    pub identity: String,
    pub retention_days: u64,
}

thread_local! {
    static SET_RETENTION_INPUT_ENCODINGS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

impl SetRetentionInput {
    /// Starts counting how many times this thread serializes a retention input.
    pub fn reset_encoding_count() {
        SET_RETENTION_INPUT_ENCODINGS.with(|count| count.set(0));
    }

    /// How many times this thread serialized a retention input since the reset.
    pub fn encoding_count() -> u32 {
        SET_RETENTION_INPUT_ENCODINGS.with(std::cell::Cell::get)
    }
}

/// Serializes exactly as a derived impl would, and counts each call so a proof
/// can show how many times a request encodes its input.
impl serde::Serialize for SetRetentionInput {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        SET_RETENTION_INPUT_ENCODINGS.with(|count| count.set(count.get() + 1));
        let mut state = serializer.serialize_struct("SetRetentionInput", 2)?;
        state.serialize_field("identity", &self.identity)?;
        state.serialize_field("retention_days", &self.retention_days)?;
        state.end()
    }
}
worth_query_portable_type!(SetRetentionInput => "worth.query.certification.document-retention.set-input.v1");
worth_query_structured_value_binding!(pub SetRetentionInputBinding for SetRetentionInput {
    identity: "worth.query.certification.document-retention.set-input.v1"
});
worth_query_operation!(pub SetRetention for DocumentRetentionSchema, input SetRetentionInputBinding);

impl primary_graph::WorthQueryApplicationProjection<DocumentRetentionSchema, DocumentRetentionQuery>
    for DocumentRetentionRow
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            DocumentRetentionSchema,
            DocumentRetentionQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(document_retention_query::identity_result())?,
            retention_days: row.field(document_retention_query::retention_result())?,
        })
    }
}

impl
    primary_graph::WorthQueryApplicationProjection<
        DocumentRetentionSchema,
        DocumentRetentionConditionQuery,
    > for bool
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            DocumentRetentionSchema,
            DocumentRetentionConditionQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(row.field(document_retention_query::condition_retention_result())? > 0)
    }
}
