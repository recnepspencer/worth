pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
pub use runtime_domain as domain;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadyDomain;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadyOperation;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadyFamily;
conditional_node!(pub(super) ReadyNode in ReadyDomain, ReadyOperation, ReadyFamily => operation "optional-aspect-ready");
pub struct InitialReady<Schema>(PhantomData<fn() -> Schema>);
pub struct KeepReady<Schema>(PhantomData<fn() -> Schema>);
pub fn definition<Schema: BirthSchema>(
) -> domain::WorthQueryDomainOperationDefinition<ReadyDomain, ReadyOperation, ReadyFamily> {
    let (projection, dependency, query) = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::readiness::source::meaning::<Schema>();
    WorthQueryOutputReadinessContractBuilder::new(
        domain::WorthQueryDomainOperationIdentity::new("optional-aspect-readiness", 1),
        "optional-aspect-ready",
        projection.clone(),
        query,
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new("optional-aspect-readiness").unwrap(),
        "optional-aspect-readiness-v1",
    )
    .semantic_reads([projection])
    .dependencies([dependency.clone()])
    .readiness_dependencies([dependency])
    .build()
    .unwrap()
}
macro_rules! ready {
    ($ready:ident, $op:ident, $producer:ident, $identity:literal, $input:ty) => {
        impl<Schema: BirthSchema> $ready<Schema> {
            fn binding() -> domain::WorthQueryApplicationConditionalOperationBinding<
                Schema,
                $op,
                $input,
                ReadyDomain,
                ReadyOperation,
                ReadyFamily,
            > {
                domain::WorthQueryApplicationConditionalOperationBinding::declare(
                    $op::reference(),
                    definition::<Schema>().reference(),
                )
            }
        }
        impl<Schema: BirthSchema> WorthQueryApplicationConditionalBinding<Schema>
            for $ready<Schema>
        {
            type Configuration = ();
            type Installed = ();
            type Operation = $op;
            const IDENTITY: &'static str = $identity;
            const REQUIRED_PRODUCERS: &'static [&'static str] =
                &[<$producer<Schema> as WorthQueryApplicationProducerBinding<Schema>>::IDENTITY];
            fn package_contract(
_declaration: &worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<WorthQueryApplicationConditionalPackageContract, worth_query_execution::facade::application_contribution::WorthQueryOutputReadinessContractDenial> {
Ok({
                WorthQueryApplicationConditionalPackageContract::new(
                    definition::<Schema>().into_portable(),
                    Self::binding().portable().clone(),
                    ReadyNode::reference().node_identity(),
                )
            })
}
            fn install(
                _: (),
                _: &WorthQueryApplicationConditionalProducerAccess<'_, Schema>,
                installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
            ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
                let operation = installation
                    .installed_schema()
                    .installed_operation($op::reference())
                    .unwrap();
                let node = installation
                    .installed_packages()
                    .bind_conditional_application_operation(operation, &Self::binding())
                    .unwrap()
                    .bind_node(ReadyNode::reference())
                    .unwrap();
                installation.bind_output_readiness::<$producer<Schema>, _, _, _, _, _, _>(node, 0)
            }
        }
    };
}
ready!(
    InitialReady,
    Produce,
    Initial,
    "optional-aspect-initial-ready",
    Input
);
ready!(
    KeepReady,
    KeepProduce,
    Preserve,
    "optional-aspect-preserve-ready",
    KeepInput
);

#[path = "source/mod.rs"]
pub mod source;
