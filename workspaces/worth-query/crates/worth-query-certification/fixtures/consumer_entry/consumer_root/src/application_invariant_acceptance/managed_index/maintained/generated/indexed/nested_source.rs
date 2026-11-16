//! Fixed output applicability and ordinary Source-derived producer input.
use crate::application_invariant_acceptance::managed_index::maintained as catalog;
use catalog::generated::operation as ordinary;
use catalog::schema::*;
pub(in crate::application_invariant_acceptance::managed_index::maintained) mod motion;
mod operation;
pub(in crate::application_invariant_acceptance::managed_index::maintained) mod program;
mod readiness;
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_host::facade::application_contribution::*;
pub struct Family;
const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "applied-nested-source",
    WorthQueryProducerLifecyclePosture::Initial,
);
impl WorthQueryProducerOutputFamily<Catalog> for Family {
    type Source = catalog::geometry::query::BoxBinding;
    type Entity = Item;
    const IDENTITY: &'static str = "maintained-generated.applied-nested-source.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &catalog::geometry::query::Row) -> &'static str {
        "applied-nested-source"
    }
}
pub struct Provider;
pub struct Producer;
impl WorthQueryApplicationProducerProvider<Catalog, Producer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str =
        "maintained-generated.applied-nested-source.provider.v2";
    fn operation_input(&self, source: &catalog::geometry::query::Row) -> ordinary::Generate {
        // This business mapping treats ASCII case variants as the same generated item.
        let name = format!("nested-{}", source.identity);
        ordinary::Generate {
            identity: format!("managed-{}-{}", source.identity, name),
            directory: "A".into(),
            name: format!("produced:{name}"),
        }
    }
    fn idempotency_key(
        &self,
        _: &catalog::geometry::query::Row,
        source_identity: &[u8; 32],
    ) -> u64 {
        u64::from_le_bytes(source_identity[..8].try_into().unwrap())
    }
}
impl WorthQueryApplicationProducerBinding<Catalog> for Producer {
    type Operation = operation::Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = ordinary::ItemOutput;
    const IDENTITY: &'static str = "maintained-generated.applied-nested-source.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "AtMostOneTakenName",
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
        )];
    const REUSE_POLICY: &'static str = "exact-source";
    const INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
}
#[derive(Clone)]
pub struct Demand {
    pub directory: String,
}
impl WorthQueryApplicationOutputDemand<Catalog> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> catalog::geometry::query::FindBoxes {
        catalog::geometry::query::FindBoxes {
            directory: self.directory.clone(),
            probe: catalog::bounds::Coordinates::new(-1, -1, 1, 1),
        }
    }
}

#[cfg(test)]
mod court;
#[cfg(test)]
mod first_publication;

pub(super) fn declare(
    builder: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        Catalog,
    >,
) -> worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Catalog> {
    motion::declare(builder).application_mutation_binding::<operation::Binding>()
}
pub(super) fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<Producer>()?;
    contracts.conditional::<readiness::Readiness>()?;
    Ok(())
}
pub(super) fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<operation::Binding, _>(operation::Handler)?;
    setup.handler::<motion::Binding, _>(motion::Handler)?;
    setup.producer::<Producer>(Provider)?;
    setup.conditional::<readiness::Readiness>(())
}
