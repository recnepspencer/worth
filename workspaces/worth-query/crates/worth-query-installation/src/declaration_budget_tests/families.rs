//! One roster generates both compiler exhaustiveness and required fixture coverage.
use std::collections::BTreeSet;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

macro_rules! declaration_families {
    ($($variant:ident => $tag:literal),+ $(,)?) => {
        pub(super) fn canonical_family(member: &ApplicationSchemaMember) -> &'static str {
            match member { $(ApplicationSchemaMember::$variant { .. } => $tag,)+ }
        }
        pub(crate) fn required() -> BTreeSet<String> {
            let tags = [$($tag.to_owned()),+];
            let required: BTreeSet<_> = tags.iter().cloned().collect();
            assert_eq!(required.len(), tags.len(), "each declaration family needs a distinct court entry");
            required
        }
    };
}

declaration_families! {
    DecisionReadScope => "decision-read-scope",
    OperationDecisionReadScope => "operation-decision-read-scope",
    ContentFormat => "content-format",
    ApplicationHistory => "application-history",
    InvariantManagedIndexSource => "invariant-managed-index-source",
    ManagedIndex => "managed-index",
    ApplicationResource => "application-resource",
    ApplicationMutation => "application-mutation",
    ApplicationInvariant => "application-invariant",
    Entity => "entity",
    Aspect => "aspect",
    Field => "field",
    Relation => "relation",
    RelationAspect => "relation-aspect",
    DerivedRelation => "derived-relation",
    RuleReadableCollection => "rule-readable-collection",
    OperationDerivedRead => "operation-derived-read",
    ComponentSet => "component-set",
    DerivedComponentSet => "derived-component-set",
    OperationComponentRead => "operation-component-read",
    PrincipalBinding => "principal-binding",
    ApplicationQuery => "application-query",
    ApplicationHistoryQuery => "application-history-query",
    ApplicationQueryInspection => "application-query-inspection",
    ApplicationCapability => "application-capability",
    ApplicationCapabilityContext => "application-capability-context",
    ApplicationCapabilityContextEntitySlot => "application-capability-context-entity-slot",
    ApplicationCapabilityProvenance => "application-capability-provenance",
    Operation => "operation",
    OperationProgram => "operation-program",
    OperationBody => "operation-body-v1",
    OperationDecisionRead => "operation-decision-read",
    OperationMutationPrecondition => "operation-mutation-precondition",
    OperationProjectionWorkBudget => "operation-projection-work-budget",
    OperationExternalEffect => "operation-external-effect",
    OperationInboundOccurrence => "operation-inbound-occurrence",
    OperationAftermath => "operation-aftermath",
    Policy => "policy",
    Ability => "ability",
    OperationAbility => "operation-ability",
    AbilityPolicy => "ability-policy",
    Unit => "unit",
    Effect => "effect",
    ProjectedMemberDomain => "projected-member-domain",
    ScalarAspectField => "scalar-aspect-field",
}
