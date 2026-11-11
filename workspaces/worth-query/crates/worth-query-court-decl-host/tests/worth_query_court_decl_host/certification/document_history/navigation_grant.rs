//! Ordinary recorded grant and actor path for generic model snapshot navigation.
macro_rules! navigation_grant {
    () => {
        use super::*;
        use worth_query_decl::facade::application_schema::fragment::*;
        #[derive(Clone, Copy)]
        pub struct GrantStorage;
        impl Fragment for GrantStorage {
            const NAME: &'static str = "grants";
            type Bindings = GrantBindings;
            type Members = (Grant,);
        }
        pub struct GrantBindings;
        impl BindingSet for GrantBindings {
            type Work<'a> = ValueDecodeAdmission<'a>;
            fn with_admission<T>(
                work: &mut Self::Work<'_>,
                run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
            ) -> T {
                run(work)
            }
        }
        impl Scalar<String> for GrantBindings {
            type Binding = StringApplicationValueBinding;
            const WRITABLE: bool = false;
        }
        impl Scalar<u64> for GrantBindings {
            type Binding = U64ApplicationValueBinding;
            const WRITABLE: bool = false;
        }
        #[derive(Clone, Copy)]
        pub struct Grant;
        impl Entity for Grant {
            const NAME: &'static str = "Grant";
            type Aspects = (Facts,);
            type Relations = (Resource, Parent);
        }
        impl Grant {
            pub fn reference() -> ApplicationEntityRef<Schema, Self> {
                entity(GrantStorage, Self)
            }
        }
        pub struct Facts;
        impl Aspect for Facts {
            const NAME: &'static str = "Facts";
            type Row = GrantRow;
        }
        pub struct GrantRow {
            pub action: String,
            pub purpose: String,
            pub status: String,
            pub workflow: String,
            pub before: u64,
            pub after: u64,
            pub depth: u64,
        }
        impl Record<GrantRow> for GrantBindings {
            fn record() -> impl Shape<GrantRow, Self> {
                field("Action", |row: &GrantRow| row.action.clone())
                    .equality_queryable(true)
                    .and("Purpose", |row: &GrantRow| row.purpose.clone())
                    .equality_queryable(true)
                    .and("Status", |row: &GrantRow| row.status.clone())
                    .equality_queryable(true)
                    .and("Workflow", |row: &GrantRow| row.workflow.clone())
                    .equality_queryable(true)
                    .and("Before", |row: &GrantRow| row.before)
                    .equality_queryable(true)
                    .and("After", |row: &GrantRow| row.after)
                    .equality_queryable(true)
                    .and("Depth", |row: &GrantRow| row.depth)
                    .equality_queryable(true)
                    .build(
                        |action,
                         purpose,
                         status,
                         workflow,
                         before,
                         after,
                         depth,
                         _: &mut ValueDecodeAdmission<'_>| {
                            Ok::<_, std::convert::Infallible>(GrantRow {
                                action,
                                purpose,
                                status,
                                workflow,
                                before,
                                after,
                                depth,
                            })
                        },
                    )
            }
        }
        pub type Action = FieldSelection<GrantStorage, Grant, Facts, String>;
        pub type Purpose = FieldSelection<GrantStorage, Grant, Facts, String>;
        pub type Status = FieldSelection<GrantStorage, Grant, Facts, String>;
        pub type Workflow = FieldSelection<GrantStorage, Grant, Facts, String>;
        pub type Before = FieldSelection<GrantStorage, Grant, Facts, u64>;
        pub type After = FieldSelection<GrantStorage, Grant, Facts, u64>;
        pub type Depth = FieldSelection<GrantStorage, Grant, Facts, u64>;
        pub struct Owner;
        impl Relation for Owner {
            const NAME: &'static str = "Owner";
            type To = Model;
        }
        impl Owner {
            pub fn reference() -> ApplicationRelationRef<Schema, Self, Principal, Model> {
                relation(Storage, Principal, Self)
            }
        }
        pub struct Resource;
        impl Relation for Resource {
            const NAME: &'static str = "Resource";
            type To = Model;
        }
        impl Resource {
            pub fn reference() -> ApplicationRelationRef<Schema, Self, Grant, Model> {
                relation(GrantStorage, Grant, Self)
            }
        }
        pub struct Parent;
        impl Relation for Parent {
            const NAME: &'static str = "Parent";
            type To = Grant;
        }
        impl Parent {
            pub fn reference() -> ApplicationRelationRef<Schema, Self, Grant, Grant> {
                relation(GrantStorage, Grant, Self)
            }
        }
        pub struct Grantor;
        impl Relation for Grantor {
            const NAME: &'static str = "Grantor";
            type To = Grant;
        }
        impl Grantor {
            pub fn reference() -> ApplicationRelationRef<Schema, Self, Principal, Grant> {
                relation(Storage, Principal, Self)
            }
        }
        pub struct Grantee;
        impl Relation for Grantee {
            const NAME: &'static str = "Grantee";
            type To = Grant;
        }
        impl Grantee {
            pub fn reference() -> ApplicationRelationRef<Schema, Self, Principal, Grant> {
                relation(Storage, Principal, Self)
            }
        }
        pub fn text(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
            ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
        }

        pub fn declare(
            schema: ApplicationSchemaDeclarationBuilder<Schema>,
        ) -> ApplicationSchemaDeclarationBuilder<Schema> {
            schema
        }
        pub fn contract(
        ) -> ApplicationCapabilityContract<Schema, Navigate, SelectSnapshot, query::model::Read> {
            let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                .forward(Owner::reference())
                .allow(Model::reference());
            ApplicationCapabilityContractBuilder::new(
                Navigate::reference(),
                SelectSnapshot::reference(),
                Grant::reference(),
            )
            .target(ApplicationCapabilityTargetDefinition::new(
                ApplicationCapabilityValueBinding::new(Action::at("Action"), text("navigate"))
                    .unwrap(),
                ApplicationCapabilityRelationBinding::from_reference(Resource::reference()),
                ApplicationCapabilityRelationDimension::not_applicable(),
                ApplicationCapabilityFieldDimension::not_applicable(),
                ApplicationCapabilityValueBinding::new(Purpose::at("Purpose"), text("history"))
                    .unwrap(),
            ))
            .constraints(ApplicationCapabilityConstraintDefinition::new(
                ApplicationCapabilityMagnitudeDimension::not_applicable(),
                ApplicationCapabilityCardinalityDimension::One,
                ApplicationCapabilityCurrentnessDefinition::new(
                    ApplicationCapabilityValueBinding::new(Status::at("Status"), text("active"))
                        .unwrap(),
                    ApplicationCapabilityWorkflowDefinition::new(
                        ApplicationCapabilityFieldBinding::from_reference(Workflow::at("Workflow"))
                            .unwrap(),
                        ApplicationCapabilityFieldBinding::from_reference(ModelKey::at("ModelKey"))
                            .unwrap(),
                    ),
                    ApplicationCapabilityValidityDefinition::new(
                        ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                        ApplicationCapabilityFieldBinding::from_reference(Before::at("Before"))
                            .unwrap(),
                        ApplicationCapabilityFieldBinding::from_reference(After::at("After"))
                            .unwrap(),
                    ),
                ),
                Context::reference(),
            ))
            .delegation(ApplicationCapabilityDelegationDefinition::new(
                ApplicationCapabilityRelationBinding::from_reference(Parent::reference()),
                ApplicationCapabilityRelationBinding::from_reference(Grantor::reference()),
                ApplicationCapabilityRelationBinding::from_reference(Grantee::reference()),
                ApplicationCapabilityFieldBinding::from_reference(Depth::at("Depth")).unwrap(),
                Provenance::reference(),
            ))
            .composition(ApplicationCapabilityComposition::new(
                ApplicationCapabilityDecisionComposition::new(
                    ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                        ApplicationCapabilityGraphClause::new(allow),
                    ])),
                    ApplicationCapabilityDenyRule::not_applicable(),
                    ApplicationCapabilityConflictRule::not_applicable(),
                ),
                ApplicationCapabilityActorComposition::new(
                    ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
                    ApplicationCapabilityDistinctActorRule::not_applicable(),
                    ApplicationCapabilityRecordedActorRule::not_applicable(),
                ),
                ApplicationCapabilityPropagationComposition::new(
                    ApplicationCapabilityDelegationRule::forbidden(),
                    ApplicationCapabilityDisclosureRule::not_applicable(),
                ),
            ))
            .elevation(ApplicationCapabilityElevationRule::not_applicable())
            .build()
        }
        pub fn seed(
            graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Grant::reference(),
                    WorthQueryApplicationEntityKey::new("navigation-grant").unwrap(),
                )
                .field(Action::at("Action"), "navigate".to_owned())
                .field(Purpose::at("Purpose"), "history".to_owned())
                .field(Status::at("Status"), "active".to_owned())
                .field(Workflow::at("Workflow"), "a".to_owned())
                .field(Before::at("Before"), 0)
                .field(After::at("After"), 4_000_000_000)
                .field(Depth::at("Depth"), 0),
            )?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Owner::reference(),
                "navigation-owner",
                WorthQueryApplicationEntityKey::new("operator").unwrap(),
                WorthQueryApplicationEntityKey::new("a").unwrap(),
            ))?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Resource::reference(),
                "navigation-resource",
                WorthQueryApplicationEntityKey::new("navigation-grant").unwrap(),
                WorthQueryApplicationEntityKey::new("a").unwrap(),
            ))?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Grantor::reference(),
                "navigation-grantor",
                WorthQueryApplicationEntityKey::new("operator").unwrap(),
                WorthQueryApplicationEntityKey::new("navigation-grant").unwrap(),
            ))?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Grantee::reference(),
                "navigation-grantee",
                WorthQueryApplicationEntityKey::new("operator").unwrap(),
                WorthQueryApplicationEntityKey::new("navigation-grant").unwrap(),
            ))?;
            Ok(())
        }
    };
}
