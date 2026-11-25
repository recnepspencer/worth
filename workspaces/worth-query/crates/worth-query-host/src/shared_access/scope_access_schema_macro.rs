//! Grant records lowered by `worth_query_scope_access!`.

#[doc(hidden)]
#[macro_export]
macro_rules! __worth_query_scope_access_schema {
    ($Schema:ident, $Scope:ident, $Principal:ident) => {
        /// Scalar bindings of the stock grant record.
        pub struct ScopeGrantBindings;
        impl BindingSet for ScopeGrantBindings {
            type Work<'a> = ValueDecodeAdmission<'a>;
            fn with_admission<T>(
                work: &mut Self::Work<'_>,
                run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
            ) -> T {
                run(work)
            }
        }
        impl Scalar<::std::string::String> for ScopeGrantBindings {
            type Binding = StringApplicationValueBinding;
            const WRITABLE: bool = true;
        }
        impl Scalar<u64> for ScopeGrantBindings {
            type Binding = U64ApplicationValueBinding;
            const WRITABLE: bool = true;
        }
        impl Scalar<bool> for ScopeGrantBindings {
            type Binding = BoolApplicationValueBinding;
            const WRITABLE: bool = true;
        }
        impl Scalar<ScopeAccess> for ScopeGrantBindings {
            type Binding = ScopeAccessBinding;
            const WRITABLE: bool = true;
        }
        impl Scalar<ScopeGrantAction> for ScopeGrantBindings {
            type Binding = ScopeGrantActionBinding;
            const WRITABLE: bool = true;
        }
        impl Scalar<ScopeGrantPurpose> for ScopeGrantBindings {
            type Binding = ScopeGrantPurposeBinding;
            const WRITABLE: bool = true;
        }
        impl Scalar<ScopeGrantDisclosure> for ScopeGrantBindings {
            type Binding = ScopeGrantDisclosureBinding;
            const WRITABLE: bool = true;
        }

        /// The grant entity, its principal relations and record.
        #[derive(Clone, Copy)]
        pub struct ScopeGrantRecords;
        impl Fragment for ScopeGrantRecords {
            const NAME: &'static str = "worth_scope_grant_records";
            type Bindings = ScopeGrantBindings;
            type Members = (
                ScopeGrant,
                Extend<$Principal, (), (ScopeGrantor, ScopeGrantee)>,
            );
        }

        /// One durable grant of typed access to one scope.
        #[derive(Clone, Copy, $crate::facade::WorthQuerySchema)]
        #[worth_query(
            entity,
            aspects(ScopeGrantFacts),
            relations(ScopeGrantResource, ScopeGrantParent),
            reference($Schema, ScopeGrantRecords)
        )]
        pub struct ScopeGrant;

        /// The principal who issued a grant.
        #[derive($crate::facade::WorthQueryRelation)]
        #[worth_query(
                    to = ScopeGrant, from = $Principal, reference($Schema, ScopeGrantRecords),
                    cross_context = ApplicationRelationCrossContextPolicy::AllowExplicit,
                )]
        pub struct ScopeGrantor;

        /// The principal a grant authorizes.
        #[derive($crate::facade::WorthQueryRelation)]
        #[worth_query(
                    to = ScopeGrant, from = $Principal, reference($Schema, ScopeGrantRecords),
                    cross_context = ApplicationRelationCrossContextPolicy::AllowExplicit,
                )]
        pub struct ScopeGrantee;

        /// The scope a grant covers.
        #[derive($crate::facade::WorthQueryRelation)]
        #[worth_query(
                    to = $Scope, from = ScopeGrant, reference($Schema, ScopeGrantRecords),
                    cross_context = ApplicationRelationCrossContextPolicy::AllowExplicit,
                )]
        pub struct ScopeGrantResource;

        /// The grant a delegated grant derives from.
        #[derive($crate::facade::WorthQueryRelation)]
        #[worth_query(
                    to = ScopeGrant, from = ScopeGrant, reference($Schema, ScopeGrantRecords),
                    cross_context = ApplicationRelationCrossContextPolicy::AllowExplicit,
                )]
        pub struct ScopeGrantParent;

        /// The grant's typed capability dimensions.
        #[derive($crate::facade::WorthQueryRecord)]
        #[worth_query(
                    bindings = ScopeGrantBindings,
                    aspect = ScopeGrantFacts,
                    fragment = ScopeGrantRecords,
                    entity = ScopeGrant
                )]
        pub struct ScopeGrantRow {
            #[worth_query(equality, unique)]
            pub grant_id: ::std::string::String,
            #[worth_query(equality)]
            pub action: ScopeGrantAction,
            #[worth_query(equality)]
            pub purpose: ScopeGrantPurpose,
            #[worth_query(equality)]
            pub disclosure: ScopeGrantDisclosure,
            #[worth_query(equality)]
            pub access: ScopeAccess,
            #[worth_query(equality)]
            pub active: bool,
            #[worth_query(equality)]
            pub workflow: ::std::string::String,
            #[worth_query(equality)]
            pub not_before: u64,
            #[worth_query(equality)]
            pub not_after: u64,
            #[worth_query(equality)]
            pub delegation_limit: u64,
        }

        /// The grant records and capability registrations; add to the
        /// application contribution's members.
        #[derive(Clone, Copy)]
        pub struct ScopeAccessMembers;
        impl Fragment for ScopeAccessMembers {
            const NAME: &'static str = "worth_scope_access_members";
            type Bindings = NoBindings;
            type Members = (ScopeGrantRecords, ScopeAccessRegistrations);
        }
    };
}
