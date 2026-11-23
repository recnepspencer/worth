//! A declared shared-owner (peer-scope) Parent read lowers fail-closed: its
//! governing relation must exist, and a peer must be named by input.
use super::decision_read::{DecisionOperation, PrincipalDecision};
use super::{bind, model::*, queries::*, SchemaKind};
use worth_query_declaration::facade::application_schema::*;

/// The governing relation's typed marker for this operation's reads.
pub struct SharedOwner;
impl OperationReads<DecisionOperation> for SharedOwner {}

pub(super) fn declare<S: ModelSchema>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
    operation: ApplicationOperationRef<S, DecisionOperation, ()>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    // Mode 10 names the real MappingTarget relation; mode 11 a relation the
    // schema never declared.
    let relation = ApplicationRelationRef::<S, SharedOwner, Mapping<S>, Principal<S>>::from_schema_identifiers(
        if S::MODE == 10 { "MappingTarget" } else { "GhostOwner" },
        "Mapping",
        "Principal",
        mapping_target::<S>().integrity(),
    );
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(operation, principal_identity::<S>())
        .operation_managed_index_read::<ParentBinding<S>, _, _>(operation)
        .operation_shared_owner_managed_index_read::<ParentBinding<S>, _, _, _, _, _>(
            operation, relation,
        )
        .application_mutation_binding::<PrincipalDecision<S>>()
}

#[test]
fn shared_owner_read_refuses_a_principal_named_peer_and_an_undeclared_relation() {
    // The governing relation exists, but the Parent and the operation resolve
    // their scope from the principal: a peer scope cannot be named, so the
    // operation does not install.
    let Err(principal_peer) = bind::<10>() else {
        panic!("a principal-scoped Parent cannot be read at a peer scope");
    };
    assert_eq!(principal_peer.kind(), SchemaKind::OperationInstallationDenied);
    assert_eq!(
        principal_peer.subject(),
        "worth.query.tests.managed-parent-principal-decision.v1"
    );
    // A governing relation the schema never declared is refused at declaration.
    assert!(
        ParentSchema::<11>::declaration().is_err(),
        "an undeclared governing relation cannot authorize a peer read"
    );
    // The own-scope twin without the peer declaration still installs.
    let lawful = bind::<0>().unwrap();
    let declared = lawful
        .installed_mutation_binding::<PrincipalDecision<ParentSchema<0>>>()
        .unwrap();
    let parent = worth_query_declaration::facade::application_query::ApplicationQueryBindingReference::of::<
        ParentSchema<0>,
        ParentBinding<ParentSchema<0>>,
    >();
    assert!(
        declared
            .operation()
            .contracts()
            .shared_owner_managed_index_reads(&parent)
            .next()
            .is_none(),
        "no peer read is installed unless declared"
    );
}
