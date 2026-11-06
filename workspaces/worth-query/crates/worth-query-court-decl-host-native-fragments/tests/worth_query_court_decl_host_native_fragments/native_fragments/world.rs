//! An installed type-declared schema, seeded only through native bootstrap.
use super::authentication::{authenticate_external, external_identity, live_scope};
use super::resources::{execution_policy, product_world_resources};
use super::*;
use std::time::Duration;
use worth_query_decl::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller;

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    None,
    Named,
    Measured,
    BothBroken,
}
pub(super) type Program<O> = WorthQueryApplicationEffectProgramBuilder<Fixture, O, (), Item>;
pub(crate) struct World {
    pub(super) application: WorthQueryPrimaryGraphApplicationRuntime<Fixture>,
    pub(super) binding: WorthQueryInstalledPrincipalBinding<
        Fixture,
        IdentityBinding,
        Mapping,
        Principal,
        u64,
        IdentityValueBinding,
    >,
    pub(super) invariant: WorthQueryApplicationInvariantProjectionAuthority<Fixture>,
}

impl World {
    pub(crate) fn new(mode: Mode) -> Self {
        worth_query_host::facade::application_contribution::with_bootstrap_advancement(
            execution_policy(),
            |phase| {
                let declaration = <Fixture as ApplicationSchema>::declaration().unwrap();
                let package =
                    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
                        Fixture::OWNER,
                        Fixture::MAJOR,
                        Fixture::MINOR,
                    ))
                    .application_schema(declaration.clone())
                    .validate()
                    .unwrap();
                let admitted =
                    WorthQueryInstallationAdmissionProfile::new("support", "configuration")
                        .admit(package)
                        .unwrap();
                let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
                    .install(WorthQueryInstallationGeneration::initial(), [admitted])
                    .unwrap()
                    .into_parts();
                let schema = runtime
                    .installed_packages()
                    .bind_application_schema(declaration)
                    .unwrap();
                let binding = schema
                    .principal_binding(IdentityBinding::reference().unwrap())
                    .unwrap();
                let resources = product_world_resources();
                let mut bootstrap = authority
                    .prepare_primary_graph(&phase, &runtime, &schema, resources)
                    .unwrap();
                bootstrap
                    .bind_principal(
                        &binding,
                        WorthQueryApplicationPrincipalKey::new("person").unwrap(),
                        1,
                        external_identity("person"),
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )
                    .unwrap();
                let mut seed = WorthQueryApplicationEntitySeed::new(
                    entity(Fixture, Item),
                    WorthQueryApplicationEntityKey::new("item").unwrap(),
                )
                .record_field(
                    record_field(Fixture, Item, ItemIdentity, "id").unwrap(),
                    11u64,
                )
                .unwrap();
                for (path, value) in [
                    ("components[0]", 2u64),
                    ("components[1]", 3),
                    ("components[2]", 5),
                    ("interval.start", 7),
                    ("interval.end", 13),
                ] {
                    seed = seed
                        .record_field(record_field(Fixture, Item, Metrics, path).unwrap(), value)
                        .unwrap();
                }
                if matches!(mode, Mode::Named | Mode::BothBroken) {
                    seed = seed
                        .record_field(
                            record_field(Fixture, Item, Named, "text").unwrap(),
                            "initial".to_owned(),
                        )
                        .unwrap();
                }
                if matches!(mode, Mode::Measured | Mode::BothBroken) {
                    for (path, value) in [
                        ("components[0]", 17u64),
                        ("components[1]", 23),
                        ("components[2]", 31),
                        ("interval.start", 41),
                        (
                            "interval.end",
                            if matches!(mode, Mode::BothBroken) {
                                1
                            } else {
                                53
                            },
                        ),
                    ] {
                        seed = seed
                            .record_field(
                                record_field(Fixture, Item, Measured, path).unwrap(),
                                value,
                            )
                            .unwrap();
                    }
                }
                bootstrap.bind_entity(seed).unwrap();
                bootstrap
                    .bind_entity(
                        WorthQueryApplicationEntitySeed::new(
                            entity(Authentication, OptionalItem),
                            WorthQueryApplicationEntityKey::new("optional").unwrap(),
                        )
                        .record_field(
                            record_field(Authentication, OptionalItem, OptionalIdentity, "id")
                                .unwrap(),
                            19u64,
                        )
                        .unwrap(),
                    )
                    .unwrap();
                bootstrap
                    .bind_relation(WorthQueryApplicationRelationSeed::new(
                        relation(Authentication, Principal, OwnsItem),
                        "owns-item",
                        WorthQueryApplicationEntityKey::new("person").unwrap(),
                        WorthQueryApplicationEntityKey::new("item").unwrap(),
                    ))
                    .unwrap();
                bootstrap
                    .bind_relation(WorthQueryApplicationRelationSeed::new(
                        relation(Authentication, Principal, OwnsOptional),
                        "owns-optional",
                        WorthQueryApplicationEntityKey::new("person").unwrap(),
                        WorthQueryApplicationEntityKey::new("optional").unwrap(),
                    ))
                    .unwrap();
                let invariant = bootstrap.retain_invariant_projection_authority();
                let application = bootstrap
                    .publish_application_runtime(
                        &phase,
                        runtime,
                        authority,
                        schema,
                        SignalConditionalEvaluationBudget::development(),
                    )
                    .unwrap();
                Self {
                    application,
                    binding,
                    invariant,
                }
            },
        )
        .unwrap()
    }

    pub(super) fn selected(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> WorthQuerySelectedProductOperation<'_, Fixture> {
        self.application
            .on_branch(self.application.current_world())
            .select(phase)
            .unwrap()
    }

    pub(crate) fn read(&self) -> Result<FormValue, FragmentDenial> {
        let request = live_scope();
        let external = authenticate_external(
            self.application.installed_schema(),
            "person",
            Duration::from_secs(60),
            &request,
        );
        self.application
            .with_host_advancement(|phase| {
                let selected = self.selected(&phase);
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        &self.binding,
                        &external,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let target = selected
                    .resolve_entity(
                        &phase,
                        select_field(Fixture, Item, ItemIdentity, "id"),
                        11u64,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let operation = self
                    .application
                    .installed_schema()
                    .installed_operation(ChangeForm::reference())
                    .unwrap();
                let admission = selected
                    .authorize_operation(
                        &phase,
                        &principal,
                        &target,
                        &operation,
                        TypedMutationPreconditions::new(),
                        &request,
                    )
                    .unwrap();
                let (result, _, _) = self
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, projected| {
                            work(|work| {
                                reader.fragment(Fixture).unwrap().read(
                                    &projected.for_fragment(),
                                    Form,
                                    work,
                                )
                            })
                        },
                        SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                result
            })
            .unwrap()
    }

    pub(super) fn optional(&self) -> Result<Option<FormValue>, FragmentDenial> {
        let request = live_scope();
        let external = authenticate_external(
            self.application.installed_schema(),
            "person",
            Duration::from_secs(60),
            &request,
        );
        self.application
            .with_host_advancement(|phase| {
                let selected = self.selected(&phase);
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        &self.binding,
                        &external,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let target = selected
                    .resolve_entity(
                        &phase,
                        select_field(Authentication, OptionalItem, OptionalIdentity, "id"),
                        19u64,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let operation = self
                    .application
                    .installed_schema()
                    .installed_operation(InspectOptional::reference())
                    .unwrap();
                let admission = selected
                    .authorize_operation(
                        &phase,
                        &principal,
                        &target,
                        &operation,
                        TypedMutationPreconditions::new(),
                        &request,
                    )
                    .unwrap();
                let (result, _, _) = self
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, projected| {
                            work(|work| {
                                reader.fragment(Authentication).unwrap().read(
                                    &projected.for_fragment(),
                                    OptionalForm,
                                    work,
                                )
                            })
                        },
                        SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                result
            })
            .unwrap()
    }
}
