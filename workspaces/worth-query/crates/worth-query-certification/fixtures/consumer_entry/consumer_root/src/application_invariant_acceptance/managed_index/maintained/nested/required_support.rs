//! Required output acquisition retains actual empty nested adjacency facts.
use super::{query, rule, schema::*};
use std::sync::Arc;
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_host::facade::application_contribution::*;
pub(super) mod directory;
pub(super) mod operation;
pub(super) mod program;
mod readiness;
pub(super) mod source;
pub struct Family;
const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "nested-item",
    WorthQueryProducerLifecyclePosture::Initial,
);
impl WorthQueryProducerOutputFamily<Catalog> for Family {
    type Source = query::PrefixBinding;
    type Entity = Item;
    const IDENTITY: &'static str = "nested-required-support.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &query::ItemRow) -> &'static str {
        "nested-item"
    }
}
pub struct Provider(pub Arc<rule::Calls>);
pub struct Producer;
impl WorthQueryApplicationProducerProvider<Catalog, Producer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "nested-required-support.provider.v1";
    fn operation_input(&self, row: &query::ItemRow) -> operation::Generate {
        self.0.acquired.lock().unwrap().push(row.clone());
        operation::Generate {
            // The two independently seeded Source scopes have distinct rows.
            // Preserve the actual request parameters in the generated input.
            directory: if row.identity == "w" { "D" } else { "A" }.into(),
            prefix: if row.identity == "w" {
                "match-unrelated"
            } else {
                "match"
            }
            .into(),
            identity: format!("generated-{}", row.identity),
            name: format!("produced:{}:{}", row.name, row.children.len()),
        }
    }
    fn idempotency_key(&self, _: &query::ItemRow, source: &[u8; 32]) -> u64 {
        u64::from_le_bytes(source[..8].try_into().unwrap())
    }
}
impl WorthQueryApplicationProducerBinding<Catalog> for Producer {
    type Operation = operation::Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = operation::ItemOutput;
    const IDENTITY: &'static str = "nested-required-support.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] =
        &[WorthQueryProducerInvariantRequirement::new(
            "NoForbiddenProjectedName",
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
    pub prefix: String,
}
impl WorthQueryApplicationOutputDemand<Catalog> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> query::FindPrefix {
        query::FindPrefix {
            directory: self.directory.clone(),
            prefix: self.prefix.clone(),
        }
    }
}
pub(super) fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<Producer>()?;
    contracts.conditional::<readiness::Readiness>()?;
    Ok(())
}
pub(super) fn configure(
    records: Arc<rule::Calls>,
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<operation::Binding, _>(operation::Handler)?;
    setup.handler::<source::Binding, _>(super::authoring::Handler)?;
    setup.handler::<directory::Binding, _>(directory::Handler)?;
    setup.producer::<Producer>(Provider(records))?;
    setup.conditional::<readiness::Readiness>(())
}
pub(super) fn declare(
    schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        Catalog,
    >,
) -> worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Catalog> {
    directory::declare(operation::declare(schema)).application_mutation_binding::<source::Binding>()
}
