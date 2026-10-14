// Authoring differs only in the literal destination, never in the host handler.
use super::binding::{BodyBinding, SourceBinding};
use super::draft::DraftBinding;
use super::query::{document_retention_query_definition, DocumentRetentionQueryBinding};
use super::schema::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    declaration::{application_program::*, application_schema::*},
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

pub struct BodyContribution<const DESTINATION: u64>;
#[derive(Clone, Copy)]
pub enum Installation {
    Native,
    Missing,
    Replacement,
}
impl<const DESTINATION: u64> ApplicationSchemaContribution<DocumentRetentionSchema<DESTINATION>>
    for BodyContribution<DESTINATION>
{
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.body-court.contribution.v1");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<DESTINATION>>,
    ) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<DESTINATION>> {
        let authored_destination = if matches!(DESTINATION, 24..=29) {
            20
        } else {
            DESTINATION
        };
        let body = SetRetention::reference::<DocumentRetentionSchema<DESTINATION>>()
            .body()
            .expect(DocumentRetentionField::reference(), 7)
            .unwrap();
        let body = if DESTINATION == 30 {
            body.in_scope(Document::reference()).unwrap()
        } else {
            body
        };
        let body = if DESTINATION >= 100 {
            body.assign_input(
                DocumentRetentionField::reference(),
                match DESTINATION {
                    101 => "identity",
                    102 => "missing",
                    _ => "retention_days",
                },
            )
            .unwrap()
        } else {
            body.assign(DocumentRetentionField::reference(), authored_destination)
                .unwrap()
        };
        let body = if matches!(DESTINATION, 20 | 21 | 23..=29) {
            let creation = body
                .create_in_scope(Audit::reference(), "audit")
                .unwrap()
                .initialize_input(AuditIdentityField::reference(), "identity")
                .unwrap()
                .initialize(AuditRetentionField::reference(), authored_destination)
                .unwrap();
            let creation = if DESTINATION == 23 {
                creation
                    .initialize_from_scope(
                        DocumentAuditSourceField::reference(),
                        AuditPriorRetentionField::reference(),
                    )
                    .unwrap()
            } else {
                creation
                    .initialize_from_scope(
                        DocumentRetentionField::reference(),
                        AuditPriorRetentionField::reference(),
                    )
                    .unwrap()
            };
            creation.finish().unwrap()
        } else {
            body.finish().unwrap()
        };
        let schema = schema
            .entity(ExternalMapping::reference())
            .entity(Principal::reference())
            .entity(Document::reference())
            .entity(Audit::reference())
            .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .aspect(Document::reference(), DocumentFacts::reference())
            .aspect(Document::reference(), DocumentDraftFacts::reference())
            .aspect(Audit::reference(), AuditFacts::reference())
            .field(
                ExternalMapping::reference(),
                ExternalIdentityField::reference(),
            )
            .field(
                ExternalMapping::reference(),
                MappingStatusField::reference(),
            )
            .field(Principal::reference(), PrincipalIdentityField::reference())
            .field(Document::reference(), DocumentIdentityField::reference())
            .field(Document::reference(), DocumentRetentionField::reference())
            .field(Document::reference(), DocumentAuditSourceField::reference())
            .field(Document::reference(), DocumentDraftNoteField::reference())
            .field(Audit::reference(), AuditIdentityField::reference())
            .field(Audit::reference(), AuditRetentionField::reference())
            .field(Audit::reference(), AuditPriorRetentionField::reference())
            .relation(
                MappingTarget::reference(),
                ExternalMapping::reference(),
                Principal::reference(),
            )
            .principal_binding(DocumentPrincipalBinding::reference())
            .relation(
                DocumentMembers::reference(),
                Document::reference(),
                Audit::reference(),
            )
            .operation(
                SetRetention::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(body)
                    .unwrap(),
            )
            .operation(
                SetAuditSource::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(
                        SetAuditSource::reference()
                            .body()
                            .assign(DocumentAuditSourceField::reference(), 8)
                            .unwrap()
                            .finish()
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .operation(
                SetDraftNote::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(
                        SetDraftNote::reference()
                            .body()
                            .set_optional_input(DocumentDraftNoteField::reference(), "note")
                            .unwrap()
                            .finish()
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .operation(
                ClearDraftNote::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(
                        ClearDraftNote::reference()
                            .body()
                            .clear_optional(DocumentDraftNoteField::reference())
                            .finish()
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .operation(
                RequireDraftNote::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish()
                    .with_body(require_draft_note_body::<DESTINATION>())
                    .unwrap(),
            )
            .operation(
                UnlinkMember::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(UnlinkMember::reference(), 32)
            .operation_read_field(
                UnlinkMember::reference(),
                DocumentIdentityField::reference(),
            )
            .operation_read_relation(UnlinkMember::reference(), DocumentMembers::reference())
            .operation_unlink(UnlinkMember::reference(), DocumentMembers::reference())
            .operation_projection_work_budget(SetRetention::reference(), 8)
            .operation_projection_work_budget(SetAuditSource::reference(), 8)
            .operation_projection_work_budget(SetDraftNote::reference(), 8)
            .operation_projection_work_budget(ClearDraftNote::reference(), 8)
            .operation_projection_work_budget(RequireDraftNote::reference(), 32)
            .operation_read_field(
                SetRetention::reference(),
                DocumentIdentityField::reference(),
            )
            .operation_read_field(
                SetAuditSource::reference(),
                DocumentIdentityField::reference(),
            )
            .operation_read_field(
                SetDraftNote::reference(),
                DocumentIdentityField::reference(),
            )
            .operation_read_field(
                ClearDraftNote::reference(),
                DocumentIdentityField::reference(),
            )
            .operation_read_field(
                RequireDraftNote::reference(),
                DocumentIdentityField::reference(),
            )
            .application_mutation_binding::<BodyBinding<DESTINATION>>()
            .application_mutation_binding::<SourceBinding<DESTINATION>>()
            .application_mutation_binding::<DraftBinding<DESTINATION, SetDraftNote>>()
            .application_mutation_binding::<DraftBinding<DESTINATION, ClearDraftNote>>()
            .application_mutation_binding::<DraftBinding<DESTINATION, RequireDraftNote>>()
            .application_mutation_binding::<super::requirement_control::UnlinkBinding<DESTINATION>>(
            )
            .application_query(document_retention_query_definition::<DESTINATION>())
            .application_query_binding::<DocumentRetentionQueryBinding<DESTINATION>>()
            .application_query(super::audit_query::audit_query_definition::<DESTINATION>())
            .application_query_binding::<super::audit_query::AuditQueryBinding<DESTINATION>>()
            .application_query(super::draft_query::draft_query_definition::<DESTINATION>())
            .application_query_binding::<super::draft_query::DraftQueryBinding<DESTINATION>>();
        super::genesis::declaration::declare::<DESTINATION>(schema)
    }
}

fn require_draft_note_body<const DESTINATION: u64>() -> ApplicationOperationBody<
    DocumentRetentionSchema<DESTINATION>,
    RequireDraftNote,
    super::draft::DraftInput,
> {
    let body = RequireDraftNote::reference::<DocumentRetentionSchema<DESTINATION>>().body();
    let body = if matches!(DESTINATION, 25..=27 | 29) {
        let operands = RequireDraftNote::reference::<DocumentRetentionSchema<DESTINATION>>()
            .requirement_operands()
            .presence("active", DocumentDraftNoteField::reference())
            .unwrap()
            .outgoing_item_count("members", DocumentMembers::reference())
            .unwrap();
        let reader = worth_foundational::expression_api::expressions()
            .reading_within(ApplicationOperationBodyRequirementExpression::PROFILE);
        let requirement = ApplicationOperationBodyRequirement::admit(
            "members-minimum",
            operands,
            Some(reader.parse("active").unwrap()),
            reader
                .parse(if DESTINATION == 26 {
                    "members >= exact_cast<UInt64>(2)"
                } else if DESTINATION == 29 {
                    "members / exact_cast<UInt64>(0) >= exact_cast<UInt64>(1)"
                } else {
                    "members >= exact_cast<UInt64>(1)"
                })
                .unwrap(),
            "members",
        )
        .unwrap();
        body.requirement(requirement).unwrap()
    } else if DESTINATION == 28 {
        let operands = RequireDraftNote::reference::<DocumentRetentionSchema<DESTINATION>>()
            .requirement_operands()
            .field("note", DocumentDraftNoteField::reference())
            .unwrap();
        let reader = worth_foundational::expression_api::expressions()
            .reading_within(ApplicationOperationBodyRequirementExpression::PROFILE);
        let requirement = ApplicationOperationBodyRequirement::admit(
            "nonempty-note",
            operands,
            Some(reader.parse("is_some(note)").unwrap()),
            reader.parse("length(unwrap(note)) > 0").unwrap(),
            "note",
        )
        .unwrap();
        body.requirement(requirement).unwrap()
    } else if DESTINATION == 24 {
        body
    } else {
        body.require_present(DocumentDraftNoteField::reference())
    };
    body.assign(DocumentAuditSourceField::reference(), 12)
        .unwrap()
        .finish()
        .unwrap()
}

impl<const DESTINATION: u64> WorthQueryApplicationContribution<DocumentRetentionSchema<DESTINATION>>
    for BodyContribution<DESTINATION>
{
    type Configuration = Installation;
    fn configure(
        install: Installation,
        setup: &mut WorthQueryApplicationContributionSetup<
            '_,
            DocumentRetentionSchema<DESTINATION>,
        >,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        match install {
            Installation::Native => {
                if DESTINATION == 30 {
                    setup
                        .operation_body::<super::genesis::binding::CreateBinding<DESTINATION>>()?;
                }
                setup.operation_body::<BodyBinding<DESTINATION>>()?;
                setup.operation_body::<SourceBinding<DESTINATION>>()?;
                setup.operation_body::<DraftBinding<DESTINATION, SetDraftNote>>()?;
                setup.operation_body::<DraftBinding<DESTINATION, ClearDraftNote>>()?;
                setup.operation_body::<DraftBinding<DESTINATION, RequireDraftNote>>()?;
                setup.handler::<super::requirement_control::UnlinkBinding<DESTINATION>, _>(
                    super::requirement_control::UnlinkHandler,
                )?;
            }
            Installation::Missing => {}
            Installation::Replacement => {
                setup.handler::<BodyBinding<DESTINATION>, _>(super::replacement::Replacement)?
            }
        }
        Ok(())
    }
}

pub struct BodyFeature;
impl<const DESTINATION: u64> ApplicationFeature<DocumentRetentionSchema<DESTINATION>>
    for BodyFeature
{
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.body-court.feature.v1";
}
pub struct BodyProgram<const DESTINATION: u64>;
impl<const DESTINATION: u64> ApplicationProgramDefinition<DocumentRetentionSchema<DESTINATION>>
    for BodyProgram<DESTINATION>
{
    type Contributions = (BodyContribution<DESTINATION>,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.body-court.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let feature =
            ApplicationFeatureSpec::root::<DocumentRetentionSchema<DESTINATION>, BodyFeature>()
                .mutation::<BodyBinding<DESTINATION>>()
                .mutation::<SourceBinding<DESTINATION>>()
                .mutation::<DraftBinding<DESTINATION, SetDraftNote>>()
                .mutation::<DraftBinding<DESTINATION, ClearDraftNote>>()
                .mutation::<DraftBinding<DESTINATION, RequireDraftNote>>()
                .mutation::<super::requirement_control::UnlinkBinding<DESTINATION>>();
        let feature = if DESTINATION == 30 {
            feature.mutation::<super::genesis::binding::CreateBinding<DESTINATION>>()
        } else {
            feature
        };
        vec![feature.finish()]
    }
}

pub fn schema<const DESTINATION: u64>(
) -> ApplicationSchemaDeclaration<DocumentRetentionSchema<DESTINATION>> {
    ApplicationSchemaDeclarationBuilder::<DocumentRetentionSchema<DESTINATION>>::for_schema()
        .contributions()
        .register::<BodyContribution<DESTINATION>>()
        .unwrap()
        .build()
        .unwrap()
}
pub fn validated<const DESTINATION: u64>(
    schema: &ApplicationSchemaDeclaration<DocumentRetentionSchema<DESTINATION>>,
) -> ValidatedApplicationProgram<DocumentRetentionSchema<DESTINATION>, BodyProgram<DESTINATION>> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema<DESTINATION>, BodyProgram<DESTINATION>>::begin()
        .with_schema(schema).validated_program().unwrap()
}
