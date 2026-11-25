//! `worth_query_shared_access!`: one declaration implements
//! [`super::WorthQuerySharedAccessProgram`] for a string-scoped program.

/// Grant idempotency keys carry this tag above the grant sequence.
pub const WORTH_QUERY_SHARED_GRANT_KEY_TAG: u64 = 0x4000_0000_0000_0000;
/// Revocation idempotency keys carry this tag above the revocation sequence.
pub const WORTH_QUERY_SHARED_REVOKE_KEY_TAG: u64 = 0x8000_0000_0000_0000;
/// Sequence bits below the grant and revocation tags.
pub const WORTH_QUERY_SHARED_SEQUENCE_MASK: u64 = 0x3fff_ffff_ffff_ffff;

/// Implements [`crate::facade::shared_access::WorthQuerySharedAccessProgram`]
/// for a program whose shared scope is a non-empty `String` key, or a
/// `scope: Newtype,` tuple struct over one.
///
/// ```text
/// worth_query_shared_access! {
///     program Program for Schema {
///         payload: (),
///         query: SharedProjectRead = |scope, _| SharedProjectRead(scope.clone()),
///         row: ProjectRecord = |value| ProjectRecord { id: value.id, name: value.name },
///         read: ReadProjectCap => ReadProjectOp,
///         watch: WatchProjectCap => WatchProjectOp,
///         grant: GrantProjectAccess = |grant| GrantProjectAccess { .. },
///         revoke: RevokeProjectAccess = |grant_id| RevokeProjectAccess { .. },
///         owner: |schema, scope| authenticate_owner(schema, scope),
///     }
/// }
/// ```
///
/// Each capability input is the scope key. Grant and revocation mutations take
/// `u64` idempotency keys: their sequence under
/// [`WORTH_QUERY_SHARED_GRANT_KEY_TAG`] or [`WORTH_QUERY_SHARED_REVOKE_KEY_TAG`].
/// `owner` authenticates the Home owner and returns
/// `Result<WorthQueryAuthenticatedExternalPrincipal<Schema>, SharedAccessDenial>`.
#[macro_export]
macro_rules! worth_query_shared_access {
    (
        program $Program:ident for $Schema:ty {
            scope: $Scope:ident,
            $($rest:tt)*
        }
    ) => {
        $crate::worth_query_shared_access!(@impl [$Program, $Schema, $Scope, [newtype $Scope]] $($rest)*);
    };
    (
        program $Program:ident for $Schema:ty {
            $($rest:tt)*
        }
    ) => {
        $crate::worth_query_shared_access!(@impl [$Program, $Schema, ::std::string::String, [string]] $($rest)*);
    };
    (
        @impl [$Program:ident, $Schema:ty, $ScopeId:ty, $mode:tt]
            payload: $Payload:ty,
            query: $Query:ty = |$scope:ident, $payload:pat_param| $query:expr,
            row: $Row:ty = |$value:ident| $row:expr,
            read: $ReadCap:ty => $ReadOp:ty,
            watch: $WatchCap:ty => $WatchOp:ty,
            grant: $Grant:ty = |$grant:ident| $grant_intent:expr,
            revoke: $Revoke:ty = |$grant_id:ident| $revoke_intent:expr,
            owner: |$schema:ident, $request_scope:ident| $owner:expr $(,)?
    ) => {
        impl $crate::facade::shared_access::WorthQuerySharedAccessProgram<$Schema> for $Program {
            type ScopeId = $ScopeId;
            type QueryPayload = $Payload;
            type Row = $Row;
            type GrantMutationIntent = $Grant;
            type RevokeMutationIntent = $Revoke;
            type QueryIntent = $Query;
            type ReadCapability = $ReadCap;
            type ReadOperation = $ReadOp;
            type ReadCapabilityInput = ::std::string::String;
            type WatchCapability = $WatchCap;
            type WatchOperation = $WatchOp;
            type WatchCapabilityInput = ::std::string::String;

            fn scope_key(scope: &$ScopeId) -> ::std::string::String {
                $crate::worth_query_shared_access!(@key $mode scope)
            }
            fn parse_scope_key(
                key: &str,
            ) -> ::core::result::Result<$ScopeId, $crate::facade::shared_access::SharedAccessDenial> {
                if key.is_empty() {
                    return ::core::result::Result::Err($crate::facade::shared_access::SharedAccessDenial::ProtocolRefused(
                        $crate::facade::shared_access::ProtocolRefusalReason::MalformedPayload,
                    ));
                }
                ::core::result::Result::Ok($crate::worth_query_shared_access!(@parse $mode key))
            }
            fn authenticate_owner(
                $schema: &$crate::facade::domain::WorthQueryInstalledApplicationSchema<$Schema>,
                $request_scope: &$crate::facade::admission::authenticated_principal::WorthQueryRequestScope,
            ) -> ::core::result::Result<
                $crate::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<$Schema>,
                $crate::facade::shared_access::SharedAccessDenial,
            > {
                $owner
            }
            fn build_grant_mutation_intent(
                $grant: &$crate::facade::shared_access::RemoteWatchContext<'_, $ScopeId>,
            ) -> ::core::result::Result<$Grant, $crate::facade::shared_access::SharedAccessDenial> {
                ::core::result::Result::Ok($grant_intent)
            }
            fn grant_idempotency_key(_: &$crate::facade::shared_access::ScopeGrantId, sequence: u64) -> u64 {
                $crate::facade::shared_access::WORTH_QUERY_SHARED_GRANT_KEY_TAG
                    | (sequence & $crate::facade::shared_access::WORTH_QUERY_SHARED_SEQUENCE_MASK)
            }
            fn build_revoke_mutation_intent(
                $grant_id: &$crate::facade::shared_access::ScopeGrantId,
            ) -> ::core::result::Result<$Revoke, $crate::facade::shared_access::SharedAccessDenial> {
                ::core::result::Result::Ok($revoke_intent)
            }
            fn revoke_idempotency_key(_: &$crate::facade::shared_access::ScopeGrantId, sequence: u64) -> u64 {
                $crate::facade::shared_access::WORTH_QUERY_SHARED_REVOKE_KEY_TAG
                    | (sequence & $crate::facade::shared_access::WORTH_QUERY_SHARED_SEQUENCE_MASK)
            }
            fn build_query_intent(
                $scope: &$ScopeId,
                $payload: &$Payload,
            ) -> ::core::result::Result<$Query, $crate::facade::shared_access::SharedAccessDenial> {
                ::core::result::Result::Ok($query)
            }
            fn read_capability() -> $crate::facade::declaration::application_capability::ApplicationCapabilityRef<$Schema, $ReadCap> {
                <$ReadCap>::reference()
            }
            fn read_operation() -> $crate::facade::declaration::application_schema::ApplicationOperationRef<
                $Schema, $ReadOp, ::std::string::String,
            > {
                <$ReadOp>::reference()
            }
            fn build_read_capability_input(scope: &$ScopeId) -> ::std::string::String {
                $crate::worth_query_shared_access!(@key $mode scope)
            }
            fn watch_capability() -> $crate::facade::declaration::application_capability::ApplicationCapabilityRef<$Schema, $WatchCap> {
                <$WatchCap>::reference()
            }
            fn watch_operation() -> $crate::facade::declaration::application_schema::ApplicationOperationRef<
                $Schema, $WatchOp, ::std::string::String,
            > {
                <$WatchOp>::reference()
            }
            fn build_watch_capability_input(scope: &$ScopeId) -> ::std::string::String {
                $crate::worth_query_shared_access!(@key $mode scope)
            }
            fn project_row(
                $value: <<<$Query as $crate::facade::declaration::application_query::ApplicationQueryIntent<$Schema>>::Binding
                    as $crate::facade::declaration::application_query::ApplicationQueryBinding<$Schema>>::ResultBinding
                    as $crate::facade::declaration::application_schema::ApplicationStructuredValueBinding>::Value,
            ) -> $Row {
                $row
            }
        }
    };
    (@key [string] $scope:ident) => { $scope.clone() };
    (@key [newtype $Scope:ident] $scope:ident) => { $scope.0.clone() };
    (@parse [string] $key:ident) => { $key.to_owned() };
    (@parse [newtype $Scope:ident] $key:ident) => { $Scope($key.to_owned()) };

}
