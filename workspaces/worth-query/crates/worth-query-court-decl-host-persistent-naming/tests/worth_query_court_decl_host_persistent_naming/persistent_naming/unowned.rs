//! Regression court for Phase 5E Slice 2 r1 Blocker 1: an `Unowned` entity kind
//! declares zero owner hops, so unscoped `reader.resolve` on a split must
//! return `Ambiguous` with empty `owners()` on every candidate — never a bogus
//! self-owner handle cast to `NoNameOwner`.
use std::sync::Mutex;

use worth_query_decl::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameSchemeBuilder, EntityNameSubject,
    NameResolutionDenial, NameSelection, NoNameOwner, OccurrencePath, Unowned, WorthQueryName,
    WorthQueryNameRole, WorthQueryNamedKind,
};
use worth_query_decl::facade::application_schema::fragment::*;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*, authentication::*,
    WorthQueryRecord, *,
};
use worth_query_host::facade::application_naming::{
    ExecutionNameResolveDenial, WorthQueryOutputBirth,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation::*,
    primary_graph::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits, operator_identity::*,
};

#[derive(Clone, Copy, Debug)]
struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "UnownedSchema";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "persistent-naming-unowned";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        visit_correction(Self, source, visitor)
    }
}
impl ApplicationSchemaComposition for Schema {
    type Contributions = (Contribution,);
}

struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("unowned.contribution.v1");
    type Members = (Storage, Actor, Registrations);
}

#[derive(Clone, Copy)]
struct Storage;
impl Fragment for Storage {
    const NAME: &'static str = "storage";
    type Bindings = Bindings;
    type Members = (Mapping, Principal, Badge);
}

struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(work: &mut Self::Work<'_>, run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T) -> T { run(work) }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings { type Binding = WorthQueryExternalPrincipalIdentityBinding; const WRITABLE: bool = false; }
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings { type Binding = WorthQueryPrincipalMappingStatusBinding; const WRITABLE: bool = true; }
impl Scalar<String> for Bindings { type Binding = StringApplicationValueBinding; const WRITABLE: bool = false; }
impl Scalar<u64> for Bindings { type Binding = U64ApplicationValueBinding; const WRITABLE: bool = true; }

#[derive(Clone, Copy)]
struct Mapping;
impl Entity for Mapping { const NAME: &'static str = "Mapping"; type Aspects = (MappingFacts,); type Relations = (MappingTarget,); }
struct MappingFacts;
impl Aspect for MappingFacts { const NAME: &'static str = "MappingFacts"; type Row = MappingFactsRow; }
struct MappingFactsRow { externalid: WorthQueryExternalPrincipalIdentity, status: WorthQueryPrincipalMappingStatus }
impl Record<MappingFactsRow> for Bindings {
    fn record() -> impl Shape<MappingFactsRow, Self> {
        field("ExternalId", |row: &MappingFactsRow| row.externalid.clone()).equality_queryable(true)
            .and("Status", |row: &MappingFactsRow| row.status).equality_queryable(true)
            .build(|externalid, status, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>(MappingFactsRow { externalid, status }))
    }
}
type ExternalId = FieldSelection<Storage, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
type Status = FieldSelection<Storage, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
struct MappingTarget;
impl Relation for MappingTarget { const NAME: &'static str = "MappingTarget"; type To = Principal; }

#[derive(Clone, Copy)]
struct Principal;
impl Entity for Principal { const NAME: &'static str = "Principal"; type Aspects = (PrincipalFacts,); type Relations = (); }
struct PrincipalFacts;
impl Aspect for PrincipalFacts { const NAME: &'static str = "PrincipalFacts"; type Row = PrincipalFactsRow; }
struct PrincipalFactsRow { principalid: u64 }
impl Record<PrincipalFactsRow> for Bindings {
    fn record() -> impl Shape<PrincipalFactsRow, Self> {
        field("PrincipalId", |row: &PrincipalFactsRow| row.principalid).equality_queryable(true).read_only()
            .build(|principalid, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>(PrincipalFactsRow { principalid }))
    }
}
type PrincipalId = FieldSelection<Storage, Principal, PrincipalFacts, u64>;

#[derive(Clone, Copy)]
struct Badge;
impl Entity for Badge {
    const NAME: &'static str = "Badge";
    type Aspects = (BadgeFacts,);
    type Relations = ();
}
impl WorthQueryNamedKind for Badge {
    type Subject = EntityNameSubject;
    type OwnerScope = Unowned;
}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Bindings, aspect = BadgeFacts, fragment = Storage, entity = Badge)]
struct BadgeFactsRow {
    #[worth_query(column = "BadgeKey", equality, unique)]
    badgekey: String,
    #[worth_query(column = "BadgeValue", equality)]
    badgevalue: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryNameRole)]
enum BadgeRole {
    #[worth_query(name = "authored")]
    Authored,
    #[worth_query(name = "piece")]
    #[depends_on(Authored)]
    Piece,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
#[worth_query(identity = "worth.tests.persistent-naming.unowned-badge-side.v1")]
struct BadgeSide {
    side: u64,
}

const BADGE_SCHEME: &str = "worth.tests.persistent-naming.unowned-badge.v1";

#[derive(Clone, Copy)]
struct Actor;
impl Actor {
    fn reference() -> ApplicationPrincipalBindingRef<Schema, Self, Mapping, Principal, u64, U64ApplicationValueBinding> {
        ApplicationPrincipalBindingRef::from_requirements(
            "Actor",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(ExternalId::at("ExternalId")),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(Status::at("Status")),
                target: ApplicationPrincipalTargetRequirement::from_relation(relation(Storage, Mapping, MappingTarget)),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(PrincipalId::at("PrincipalId")),
            },
        )
    }
}
impl Registration<Schema> for Actor {
    fn register(builder: ApplicationSchemaDeclarationBuilder<Schema>) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial> {
        Ok(builder.principal_binding(Self::reference()))
    }
}

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
struct SplitIntent {
    scope: String,
    phase: u64,
}
worth_query_structured_value_binding!(SplitInput for SplitIntent { identity: "unowned.split.input.v1" });
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
struct Done;
worth_query_structured_value_binding!(DoneBinding for Done { identity: "unowned.done.v1" });
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
struct Denied(String);
worth_query_structured_value_binding!(DeniedBinding for Denied { identity: "unowned.denied.v1" });

worth_query_operation!(SplitAndProbe for Schema, input SplitInput; access {
    read_entities: [Badge];
    read_fields: [BadgeKey::at("BadgeKey"), BadgeValue::at("BadgeValue")];
    creates: [Badge];
    deletes: [Badge];
    writes: [BadgeKey::at("BadgeKey"), BadgeValue::at("BadgeValue")];
});
fn split_scope(intent: &SplitIntent) -> String {
    intent.scope.clone()
}
worth_query_mutation_binding!(SplitBinding for SplitIntent, schema Schema,
    identity "unowned.split.v1", input SplitInput, operation SplitAndProbe, result DoneBinding,
    idempotency String, identity "unowned.split.key.v1",
    decision Option<(WorthQueryInvariantMutationTarget<Schema, Badge>, WorthQueryName<Badge>)>,
    denial DeniedBinding,
    handler identity "unowned.split.handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope BadgeKey, field BadgeKey::at("BadgeKey"), value split_scope,
);

static HELD_BADGE: Mutex<Option<WorthQueryName<Badge>>> = Mutex::new(None);
static OBSERVED_OWNERS: Mutex<Option<Vec<(WorthQueryName<Badge>, usize)>>> = Mutex::new(None);

struct SplitHandler;
impl OperationHandler<Schema, SplitBinding> for SplitHandler {
    fn decide(
        &self,
        input: &SplitIntent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, SplitBinding>,
    ) -> HandlerResult<Option<(WorthQueryInvariantMutationTarget<Schema, Badge>, WorthQueryName<Badge>)>, Denied> {
        let result = (|| {
            if input.phase == 0 {
                let base = reader.resolve_entity(BadgeKey::at("BadgeKey"), "base".to_owned())?;
                let name = reader
                    .reader()
                    .name_of(&base)
                    .map_err(|cause| HandlerExecutionDenial::new(std::io::Error::other(format!("{cause:?}"))))?;
                *HELD_BADGE.lock().unwrap() = Some(name);
                return Ok(Some((reader.mutation_target(&base)?, name)));
            }
            let held = HELD_BADGE.lock().unwrap().expect("phase 0 stored base name");
            match reader
                .reader()
                .resolve(&held, &NameSelection::Exact)
                .map_err(|cause| HandlerExecutionDenial::new(std::io::Error::other(format!("{cause:?}"))))?
            {
                Err(ExecutionNameResolveDenial::Resolution(NameResolutionDenial::Ambiguous { candidates })) => {
                    let summary = candidates
                        .iter()
                        .map(|candidate| {
                            let owners: &[WorthQueryInvariantEntityIdentity<Schema, NoNameOwner>] =
                                candidate.owners();
                            (candidate.name(), owners.len())
                        })
                        .collect();
                    *OBSERVED_OWNERS.lock().unwrap() = Some(summary);
                    Ok(None)
                }
                other => Err(HandlerExecutionDenial::new(std::io::Error::other(format!(
                    "expected Ambiguous for split Unowned badge, got {other:?}"
                )))),
            }
        })();
        match result {
            Ok(Some(decision)) => HandlerResult::Completed(Some(decision)),
            Ok(None) => HandlerResult::DomainDenied(Denied("ambiguous".into())),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        _: &SplitIntent,
        decision: Option<(WorthQueryInvariantMutationTarget<Schema, Badge>, WorthQueryName<Badge>)>,
        writer: &mut CandidateWriter<'_, Schema, SplitBinding>,
    ) -> HandlerResult<Done, Denied> {
        let Some((base, base_name)) = decision else {
            return HandlerResult::Completed(Done);
        };
        let result = (|| -> Result<(), WorthQueryApplicationAttemptDenial> {
            let base_ref = writer.projected_entity(&base)?;
            let mut mk = |key: &str, val: u64| -> Result<_, WorthQueryApplicationAttemptDenial> {
                let e = writer.create_entity_in_context(
                    &base_ref,
                    entity(Storage, Badge),
                    WorthQueryApplicationEntityKey::new(key).unwrap(),
                )?;
                writer.initialize_field(&e, BadgeKey::at("BadgeKey"), key.to_owned())?;
                writer.initialize_field(&e, BadgeValue::at("BadgeValue"), val)?;
                Ok(e)
            };
            let (p1, p2) = (mk("p1", 1)?, mk("p2", 2)?);
            writer.delete_entity(entity(Storage, Badge), &base_ref)?;
            let occ = OccurrencePath::root("occ:split").expect("valid occurrence path");
            writer.split_entity_name(
                base_name,
                vec![
                    (&p1, WorthQueryOutputBirth::of_role(&occ, BadgeRole::Piece).with_discriminator(&BadgeSide { side: 1 })),
                    (&p2, WorthQueryOutputBirth::of_role(&occ, BadgeRole::Piece).with_discriminator(&BadgeSide { side: 2 })),
                ],
            )
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Done),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

struct Registrations;
impl Registration<Schema> for Registrations {
    fn register(schema: ApplicationSchemaDeclarationBuilder<Schema>) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial> {
        let scheme = ApplicationNameSchemeBuilder::<Schema, Badge>::from_roles::<BadgeRole>(BADGE_SCHEME)
            .split_discriminator::<BadgeSide>()
            .require_authored_occurrence()
            .build()
            .expect("badge scheme is well-formed");
        schema
            .declared_operation::<SplitAndProbe>(SplitAndProbe::reference().definition().no_external_effect().no_aftermath().finish())
            .application_mutation_binding::<SplitBinding>()
            .application_name_scheme::<Badge>(scheme)
    }
}

struct Entry;
impl ApplicationFeature<Schema> for Entry { type Inputs = ApplicationFeatureInputLeaf; const IDENTITY: &'static str = "unowned.entry.v1"; }
struct Completion;
impl<B: ApplicationMutationBinding<Schema>> ApplicationActionDelivery<Schema, B> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "unowned.completion.v1";
}
struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new("unowned.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Entry>().action::<SplitBinding, Completion>().finish_action().finish()]
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(_: (), setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<SplitBinding, _>(SplitHandler)
    }
}

#[test]
fn ambiguous_resolve_on_an_unowned_entity_kind_reports_empty_owners() {
    *HELD_BADGE.lock().unwrap() = None;
    *OBSERVED_OWNERS.lock().unwrap() = None;
    let limits = host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin().validated_program().unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("91820000-0000-0000-0000-00000000b71e").unwrap(),
        limits,
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(Actor::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for key in ["anchor", "base"] {
                let seed = WorthQueryApplicationEntitySeed::new(
                    entity(Storage, Badge),
                    WorthQueryApplicationEntityKey::new(key).unwrap(),
                )
                .field(BadgeKey::at("BadgeKey"), key.to_owned())
                .field(BadgeValue::at("BadgeValue"), 0)
                .persistent_name("authored");
                graph.bind_entity(seed)?;
            }
            Ok(())
        },
    )
    .expect("unowned badge world installs");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let branch = host.current_world();
    for (scope_key, phase, key) in [("base", 0, "unowned-split"), ("anchor", 1, "unowned-probe")] {
        let out = host
            .request(&principal, &scope)
            .on_branch(branch)
            .advanced()
            .mutate(SplitIntent { scope: scope_key.into(), phase })
            .without_source()
            .idempotency(&key.to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        if phase == 0 {
            assert!(matches!(out, WorthQueryApplicationMutationOutcome::Committed { .. }), "{out:?}");
        } else {
            assert!(matches!(out, WorthQueryApplicationMutationOutcome::DomainDenied(_)), "{out:?}");
        }
    }
    let candidates = OBSERVED_OWNERS.lock().unwrap().take().expect("phase 1 observed ambiguous split");
    assert_eq!(candidates.len(), 2, "{candidates:?}");
    for (name, owner_count) in &candidates {
        assert_eq!(*owner_count, 0, "Unowned candidate {name:?} must report 0 owners");
    }
}
