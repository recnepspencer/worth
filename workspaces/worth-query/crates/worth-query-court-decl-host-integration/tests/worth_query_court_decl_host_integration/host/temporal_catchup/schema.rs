//! Authored desktop reminder schema entities, aspects, records, and principal binding.

mod operations;
pub use operations::*;

use worth_query_decl::facade::{
    application_schema::{
        fragment::{self, *},
        *,
    },
    authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
        WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
    },
};

pub struct DesktopReminderSchema;
impl Fragment for DesktopReminderSchema {
    const NAME: &'static str = "DesktopReminderSchema";
    type Bindings = NoBindings;
    type Members = (DesktopReminderContribution,);
}
impl ApplicationSchema for DesktopReminderSchema {
    const OWNER: &'static str = "desktop_reminder_catchup_court";
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
impl ApplicationSchemaComposition for DesktopReminderSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct DesktopReminderContribution;
impl ApplicationSchemaContribution<DesktopReminderSchema> for DesktopReminderContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.court.desktop-reminder.v1");
    type Members = (
        ReminderModel,
        ReminderPrincipalBinding,
        (
            FireReminder,
            MutateReminder,
            super::adapters::MutateReminderBinding,
        ),
        (ReminderTargetsQuery, ReminderTargetsReadBinding),
    );
}

pub struct ReminderModel;
impl Fragment for ReminderModel {
    const NAME: &'static str = "reminder_model";
    type Bindings = ReminderModelBindings;
    type Members = (ExternalMapping, Principal, ReminderIntent);
}
pub struct ReminderModelBindings;
impl BindingSet for ReminderModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for ReminderModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for ReminderModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for ReminderModelBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = true;
}
impl Scalar<String> for ReminderModelBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = true;
}

pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
pub struct ReminderIntent;
impl Entity for ReminderIntent {
    const NAME: &'static str = "ReminderIntent";
    type Aspects = (ReminderFacts,);
    type Relations = (ReminderTarget,);
}
impl ReminderIntent {
    pub fn reference() -> ApplicationEntityRef<DesktopReminderSchema, Self> {
        entity(ReminderModel, Self)
    }
}

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<DesktopReminderSchema, Self, ExternalMapping, Principal> {
        relation(ReminderModel, ExternalMapping, Self)
    }
}

pub struct ReminderTarget;
impl Relation for ReminderTarget {
    const NAME: &'static str = "ReminderTarget";
    type To = ReminderIntent;
}
impl ReminderTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<DesktopReminderSchema, Self, ReminderIntent, ReminderIntent> {
        relation(ReminderModel, ReminderIntent, Self)
    }
}

pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
pub struct ExternalIdentityRow {
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityRow> for ReminderModelBindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityRow| {
            row.external_identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &ExternalIdentityRow| {
            row.mapping_status
        })
        .equality_queryable(true)
        .build(
            |external_identity, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityRow {
                    external_identity,
                    mapping_status,
                })
            },
        )
    }
}

pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsRow;
}
pub struct PrincipalFactsRow {
    pub principal_id: u64,
}
impl Record<PrincipalFactsRow> for ReminderModelBindings {
    fn record() -> impl Shape<PrincipalFactsRow, Self> {
        field("PrincipalIdentityField", |row: &PrincipalFactsRow| {
            row.principal_id
        })
        .equality_queryable(true)
        .read_only()
        .build(|principal_id, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalFactsRow { principal_id })
        })
    }
}

pub struct ReminderFacts;
impl Aspect for ReminderFacts {
    const NAME: &'static str = "ReminderFacts";
    type Row = ReminderFactsRow;
}
pub struct ReminderFactsRow {
    pub identity: String,
    pub revision: u64,
    pub due: u64,
    pub interval: u64,
    pub max_fires: u64,
    pub fired_count: u64,
    pub lifecycle: String,
    pub gate: String,
}
impl Record<ReminderFactsRow> for ReminderModelBindings {
    fn record() -> impl Shape<ReminderFactsRow, Self> {
        field("ReminderIdentity", |row: &ReminderFactsRow| {
            row.identity.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("ReminderRevision", |row: &ReminderFactsRow| row.revision)
        .equality_queryable(true)
        .and("ReminderDue", |row: &ReminderFactsRow| row.due)
        .equality_queryable(true)
        .and("ReminderInterval", |row: &ReminderFactsRow| row.interval)
        .equality_queryable(true)
        .and("ReminderMaxFires", |row: &ReminderFactsRow| row.max_fires)
        .equality_queryable(true)
        .and("ReminderFiredCount", |row: &ReminderFactsRow| {
            row.fired_count
        })
        .equality_queryable(true)
        .and("ReminderLifecycle", |row: &ReminderFactsRow| {
            row.lifecycle.clone()
        })
        .equality_queryable(true)
        .and("ReminderGate", |row: &ReminderFactsRow| row.gate.clone())
        .equality_queryable(true)
        .build(
            |identity,
             revision,
             due,
             interval,
             max_fires,
             fired_count,
             lifecycle,
             gate,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ReminderFactsRow {
                    identity,
                    revision,
                    due,
                    interval,
                    max_fires,
                    fired_count,
                    lifecycle,
                    gate,
                })
            },
        )
    }
}

pub type ReminderStringField = FieldSelection<ReminderModel, ReminderIntent, ReminderFacts, String>;
pub type ReminderU64Field = FieldSelection<ReminderModel, ReminderIntent, ReminderFacts, u64>;

pub fn identity_field() -> ReminderStringField {
    ReminderStringField::at("ReminderIdentity")
}
pub fn revision_field() -> ReminderU64Field {
    ReminderU64Field::at("ReminderRevision")
}
pub fn due_field() -> ReminderU64Field {
    ReminderU64Field::at("ReminderDue")
}
pub fn interval_field() -> ReminderU64Field {
    ReminderU64Field::at("ReminderInterval")
}
pub fn max_fires_field() -> ReminderU64Field {
    ReminderU64Field::at("ReminderMaxFires")
}
pub fn fired_count_field() -> ReminderU64Field {
    ReminderU64Field::at("ReminderFiredCount")
}
pub fn lifecycle_field() -> ReminderStringField {
    ReminderStringField::at("ReminderLifecycle")
}
pub fn gate_field() -> ReminderStringField {
    ReminderStringField::at("ReminderGate")
}

pub struct ReminderPrincipalBinding;
impl ReminderPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        DesktopReminderSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "ReminderPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    FieldSelection::<
                        ReminderModel,
                        ExternalMapping,
                        ExternalIdentity,
                        WorthQueryExternalPrincipalIdentity,
                    >::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    FieldSelection::<
                        ReminderModel,
                        ExternalMapping,
                        ExternalIdentity,
                        WorthQueryPrincipalMappingStatus,
                    >::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    FieldSelection::<ReminderModel, Principal, PrincipalFacts, u64>::at(
                        "PrincipalIdentityField",
                    ),
                ),
            },
        )
    }
}
impl Registration<DesktopReminderSchema> for ReminderPrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DesktopReminderSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}
