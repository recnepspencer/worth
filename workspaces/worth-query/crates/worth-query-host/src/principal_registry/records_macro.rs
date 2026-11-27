//! Account record, principal binding and onboarding requirements lowered by
//! `worth_query_principal_registry!`.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_principal_registry_records {
    ($Schema:ident, $Context:ident, $Key:ty, $key_identity:expr, $Denial:ty,
     $Principal:ident, $PrincipalKey:ident, $PrincipalFacts:ident :: $principal_id:ident,
     $Directory:ident, $Onboarding:ident) => {
        /// Scalar bindings of the account record. The external identity is
        /// never rewritten; only the status changes.
        pub struct RegistryBindings;
        impl BindingSet for RegistryBindings {
            type Work<'a> = ValueDecodeAdmission<'a>;
            fn with_admission<T>(
                work: &mut Self::Work<'_>,
                run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
            ) -> T {
                run(work)
            }
        }
        impl Scalar<WorthQueryExternalPrincipalIdentity> for RegistryBindings {
            type Binding = WorthQueryExternalPrincipalIdentityBinding;
            const WRITABLE: bool = false;
        }
        impl Scalar<WorthQueryPrincipalMappingStatus> for RegistryBindings {
            type Binding = WorthQueryPrincipalMappingStatusBinding;
            const WRITABLE: bool = true;
        }

        /// The account entity and its relation to the principal.
        #[derive(Clone, Copy)]
        pub struct RegistryRecords;
        impl Fragment for RegistryRecords {
            const NAME: &'static str = "worth_principal_registry_records";
            type Bindings = RegistryBindings;
            type Members = (PrincipalAccount, Extend<$Principal, (), (PrincipalDirectory,)>);
        }

        /// One external identity's account: maps it to exactly one principal.
        #[derive(Clone, Copy, $crate::facade::WorthQuerySchema)]
        #[worth_query(
            entity,
            aspects(PrincipalAccountFacts),
            relations(PrincipalAccountTarget),
            reference($Schema, RegistryRecords)
        )]
        pub struct PrincipalAccount;

        /// The principal an account maps to.
        #[derive($crate::facade::WorthQueryRelation)]
        #[worth_query(to = $Principal, from = PrincipalAccount, reference($Schema, RegistryRecords))]
        pub struct PrincipalAccountTarget;

        /// The one directory a registered principal belongs to: its account's
        /// status changes name that directory and are confined to it.
        #[derive($crate::facade::WorthQueryRelation)]
        #[worth_query(
            to = $Directory, from = $Principal, reference($Schema, RegistryRecords),
            cross_context = ApplicationRelationCrossContextPolicy::AllowExplicit,
        )]
        pub struct PrincipalDirectory;

        /// The account's identity is `unique`: the platform refuses a second
        /// account for one identity at commit, however the write arrives.
        #[derive($crate::facade::WorthQueryRecord)]
        #[worth_query(
            bindings = RegistryBindings,
            aspect = PrincipalAccountFacts,
            fragment = RegistryRecords,
            entity = PrincipalAccount
        )]
        pub struct PrincipalAccountRow {
            #[worth_query(equality, unique, read_only)]
            pub external_identity: WorthQueryExternalPrincipalIdentity,
            #[worth_query(equality)]
            pub status: WorthQueryPrincipalMappingStatus,
        }

        $crate::facade::worth_query_principal_binding! {
            pub Registry in $Schema, mapping PrincipalAccount => $Principal {
                identity: PrincipalAccountFacts::external_identity(),
                status: PrincipalAccountFacts::status(),
                target: PrincipalAccountTarget,
                principal: $PrincipalKey = $PrincipalFacts::$principal_id(),
            }
            context pub $Context {
                idempotency: $Key = $key_identity,
                denial: $Denial,
            }
        }

        /// Seeds one principal and its account through `bind_principal`, then
        /// enrolls the principal in `directory`: the shape `OnboardPrincipal`
        /// commits at runtime. `principal_key` is the principal's entity key.
        pub fn seed_principal(
            graph: &mut $crate::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<$Schema>,
            installed: &$crate::facade::domain::WorthQueryInstalledApplicationSchema<$Schema>,
            directory: &str,
            principal_key: &str,
            principal: <$PrincipalKey as RecordSelection>::Value,
            identity: WorthQueryExternalPrincipalIdentity,
            status: WorthQueryPrincipalMappingStatus,
        ) -> ::core::result::Result<(), $crate::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
            use $crate::facade::primary_graph::{
                WorthQueryApplicationEntityKey as Key, WorthQueryApplicationPrincipalKey as PrincipalKey,
                WorthQueryApplicationRelationSeed as RelationSeed,
                WorthQueryPrimaryGraphInstallationDenial as Denial,
                WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
            };
            let invalid = |value: &str| Denial::new(DenialKind::InvalidSchemaMember, value);
            let binding = installed
                .principal_binding(Registry::reference())
                .map_err(|denial| Denial::binding_not_installed(denial.binding()))?;
            let key = PrincipalKey::new(principal_key).map_err(|_| invalid(principal_key))?;
            graph.bind_principal(&binding, key, principal, identity, status)?;
            graph.bind_relation(RelationSeed::new(
                PrincipalDirectory::reference(),
                ::std::format!("principal-directory-{principal_key}"),
                Key::new(principal_key).map_err(|_| invalid(principal_key))?,
                Key::new(directory).map_err(|_| invalid(directory))?,
            ))
        }

        impl OperationRequiresAbility<OnboardPrincipalOperation> for $Onboarding {}
        impl OperationRequiresAbility<SetPrincipalStatusOperation> for $Onboarding {}

        /// Onboarding and status changes require the `onboarding` ability on
        /// the directory they name.
        pub struct RegistryOnboarding;
        impl Registration<$Schema> for RegistryOnboarding {
            fn register(
                builder: ApplicationSchemaDeclarationBuilder<$Schema>,
            ) -> ::core::result::Result<
                ApplicationSchemaDeclarationBuilder<$Schema>,
                ApplicationSchemaDeclarationDenial,
            > {
                ::core::result::Result::Ok(
                    builder
                        .operation_requires_ability(
                            OnboardPrincipalOperation::reference(),
                            $Onboarding::reference(),
                        )
                        .operation_requires_ability(
                            SetPrincipalStatusOperation::reference(),
                            $Onboarding::reference(),
                        ),
                )
            }
        }

        /// The account records, principal binding and onboarding
        /// requirements; add to the application contribution's members.
        #[derive(Clone, Copy)]
        pub struct RegistryMembers;
        impl Fragment for RegistryMembers {
            const NAME: &'static str = "worth_principal_registry_members";
            type Bindings = NoBindings;
            type Members = (RegistryRecords, Registry, RegistryOnboarding);
        }
    };
}
