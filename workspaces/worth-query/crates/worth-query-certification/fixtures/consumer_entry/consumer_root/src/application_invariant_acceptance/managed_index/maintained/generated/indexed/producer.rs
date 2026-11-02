//! Fixed output applicability and ordinary Source-derived producer input.
use super::super::super::query;
use super::super::super::schema::*;
use super::super::operation as ordinary;
use super::operation;
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_host::facade::application_contribution::*;
pub struct Family;
const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "indexed-catalog",
    WorthQueryProducerLifecyclePosture::Initial,
);
impl WorthQueryProducerOutputFamily<Catalog> for Family {
    type Source = query::PrefixBinding;
    type Entity = Item;
    const IDENTITY: &'static str = "maintained-generated.indexed.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &query::ItemRow) -> &'static str {
        "indexed-catalog"
    }
}
pub struct Provider;
pub struct Producer;
impl WorthQueryApplicationProducerProvider<Catalog, Producer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "maintained-generated.indexed.provider.v2";
    fn operation_input(&self, source: &query::ItemRow) -> ordinary::Generate {
        // This business mapping treats ASCII case variants as the same generated item.
        let name = source.name.to_ascii_lowercase();
        ordinary::Generate {
            identity: format!("managed-{}-{}", source.identity, name),
            directory: "A".into(),
            name: format!("produced:{name}"),
        }
    }
    fn idempotency_key(&self, _: &query::ItemRow, source_identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(source_identity[..8].try_into().unwrap())
    }
}
impl WorthQueryApplicationProducerBinding<Catalog> for Producer {
    type Operation = operation::Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = ordinary::ItemOutput;
    const IDENTITY: &'static str = "maintained-generated.indexed.producer.v1";
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
    fn source_intent(&self) -> query::FindPrefix {
        query::FindPrefix {
            directory: self.directory.clone(),
            prefix: "sh".into(),
        }
    }
}
