//! Missing scheduling or source-read grants refuse genuine factory installation.
use crate::domain_computation::{
    execution_runtime::*,
    primary_graph::{application_invariant::*, schema_layout::WorthQueryPrimaryGraphLayout, *},
};
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::*;
struct BindingSchema<const PROFILE: u8>;
struct Node;
impl<const P: u8> ApplicationEntityMarkerIdentity<BindingSchema<P>> for Node {
    const IDENTIFIER: &'static str = "Node";
}
struct Coupling;
struct Set;
struct Rule;
impl<const P: u8> ApplicationInvariantMarkerIdentity<BindingSchema<P>> for Rule {
    const IDENTIFIER: &'static str = "component-policy";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
fn set<const P: u8>() -> ApplicationComponentSetRef<BindingSchema<P>, Set, Node> {
    ApplicationComponentSetRef::new("Set", ApplicationEntityRef::from_schema_identifier("Node"))
}
impl<const P: u8> ApplicationSchema for BindingSchema<P> {
    const OWNER: &'static str = "component-resolver-test";
    const NAME: &'static str = "BindingSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let node = ApplicationEntityRef::<Self, Node>::from_schema_identifier("Node");
        let relation = |name| {
            ApplicationRelationRef::<Self, Coupling, Node, Node>::from_schema_identifiers(
                name,
                "Node",
                "Node",
                ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            )
        };
        let a = relation("A");
        let b = relation("B");
        let mut reads = vec![
            ApplicationInvariantScopeTarget::Entity("Node".into()),
            ApplicationInvariantScopeTarget::Relation("A".into()),
            ApplicationInvariantScopeTarget::Relation("B".into()),
        ];
        let mut affected = reads.clone();
        match P {
            0 => {}
            1 => affected
                .retain(|target| !matches!(target, ApplicationInvariantScopeTarget::Entity(_))),
            2 => affected
                .retain(|target| target != &ApplicationInvariantScopeTarget::Relation("B".into())),
            3 => reads
                .retain(|target| target != &ApplicationInvariantScopeTarget::Relation("B".into())),
            _ => unreachable!(),
        }
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(node)
            .relation(a, node, node)
            .relation(b, node, node)
            .components(
                ApplicationComponentSetDefinition::new(set())
                    .coupled_by(a)
                    .coupled_by(b),
            )
            .invariant(ApplicationInvariantDefinition::new(
                Rule::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
                RelationalInvariantWorkBudget::new(None),
                ApplicationInvariantOperationalContract::new(
                    ApplicationInvariantEnforcement::BlockCommit,
                    [ApplicationInvariantGroup::SchemaCompliance],
                    reads,
                    affected,
                    "primary-relational-provider",
                    ApplicationInvariantCostPosture::Global,
                ),
            ))
            .build()
    }
}
impl<const P: u8> WorthQueryApplicationInvariantRule<BindingSchema<P>> for Rule {
    type Scope = ();
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    fn prepare_scope(
        &self,
        _: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, BindingSchema<P>>,
    ) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, BindingSchema<P>>,
        _: &(),
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
fn lower<const P: u8>() -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let declaration = BindingSchema::<P>::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "component-resolver-test",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("test", "test")
        .admit(package)
        .unwrap();
    let (runtime, _) = WorthQueryExecutionRuntimeInstaller::new()
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap()
        .into_parts();
    let schema = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    let (layout, _) = WorthQueryPrimaryGraphLayout::lower(
        schema.installed_declaration(),
        schema.native_contracts(),
        &worth_relational::facade::schema::RelationalSchemaRegistry::new(),
    )
    .unwrap();
    let invariant = schema
        .installed_invariant(
            Rule::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )
        .unwrap();
    let mut factories = WorthQueryApplicationInvariantFactories::for_installed_schema(&schema);
    factories
        .bind(&invariant, |resolver| {
            resolver
                .typed_component_set(set())
                .ok_or("actual component member/source read and scheduling grants missing")?;
            Ok(Rule)
        })
        .unwrap();
    factories
        .lower(schema.binding_identity(), &layout)
        .map(|_| ())
}
#[test]
fn typed_component_factory_requires_member_and_all_sources_in_reads_and_applicability() {
    lower::<0>().unwrap();
    for error in [lower::<1>(), lower::<2>(), lower::<3>()] {
        assert_eq!(
            error.unwrap_err().kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::InvariantFactoryRejected
        );
    }
}
