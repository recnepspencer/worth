//! Window and bookmark controls have distinct typed actions and grant rows.
use super::super::*;
use super::permission::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_capability::*, application_history::*, application_schema::*,
    portable_identity::WorthQueryPortableType, worth_query_operation, worth_query_operation_reads,
};
macro_rules! policy {
    ($capability:ident, $operation:ident, $binding:ident, $role:ident, $action:literal) => {
        pub struct $capability<S>(PhantomData<fn() -> S>);
        impl<S> WorthQueryPortableType for $capability<S> { const PORTABLE_TYPE_NAME: &'static str = stringify!($capability); }
        impl<S: TopologySchemaBinding> ApplicationCapabilityMarkerIdentity for $capability<S> {
            type Schema = S;
            const IDENTIFIER: &'static str = stringify!($capability);
        }
        impl<S: TopologySchemaBinding> HistoryPermission<S> for $capability<S> {
            const ACTION: &'static str = $action;
            const GRANT_KIND: &'static str = $action;
        }
        worth_query_operation!(pub $operation for Schema: TopologySchemaBinding, input PlanarOutputReadInputBinding);
        worth_query_operation_reads!($operation => [BodyKey]);
        pub struct $binding<S>(PhantomData<fn() -> S>);
        impl<S: TopologySchemaBinding> ApplicationHistoryRetentionBinding<S, PlanarDocumentHistory<S>> for $binding<S> {
            const IDENTITY: &'static str = stringify!($binding);
            type Capability = $capability<S>;
            type Input = PlanarOutputRead;
            type InputBinding = PlanarOutputReadInputBinding;
            type Operation = $operation;
            fn input(access: &PlanarOutputRead) -> PlanarOutputRead { access.clone() }
        }
        impl<S: TopologySchemaBinding> $role<S, PlanarDocumentHistory<S>> for $binding<S> {}
    };
}
policy!(
    ChangePlanarHistoryWindow,
    ChangePlanarHistoryWindowOperation,
    PlanarHistoryWindow,
    ApplicationHistoryWindowBinding,
    "window"
);
policy!(
    ChangePlanarHistoryBookmark,
    ChangePlanarHistoryBookmarkOperation,
    PlanarHistoryBookmark,
    ApplicationHistoryBookmarkBinding,
    "bookmark"
);
pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .operation(
            ChangePlanarHistoryWindowOperation::reference::<S>()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(ChangePlanarHistoryWindowOperation::reference::<S>(), 512)
        .operation_read_field(
            ChangePlanarHistoryWindowOperation::reference::<S>(),
            BodyKey::reference(),
        )
        .capability(contract::<
            S,
            ChangePlanarHistoryWindow<S>,
            ChangePlanarHistoryWindowOperation,
        >())
        .operation(
            ChangePlanarHistoryBookmarkOperation::reference::<S>()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(
            ChangePlanarHistoryBookmarkOperation::reference::<S>(),
            512,
        )
        .operation_read_field(
            ChangePlanarHistoryBookmarkOperation::reference::<S>(),
            BodyKey::reference(),
        )
        .capability(contract::<
            S,
            ChangePlanarHistoryBookmark<S>,
            ChangePlanarHistoryBookmarkOperation,
        >())
}
