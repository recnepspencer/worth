//! Fixed output applicability and ordinary Source-derived producer input.
use super::super::schema::*;
use super::{operation, source};
use worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint;
use worth_query_host::facade::application_contribution::*;
pub struct Family;
const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "generated-catalog",
    WorthQueryProducerLifecyclePosture::Initial,
);
impl WorthQueryProducerOutputFamily<Catalog> for Family {
    type Source = source::Binding;
    type Entity = Item;
    const IDENTITY: &'static str = "maintained-generated.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL];
    fn profile_kind(_: &source::Row) -> &'static str {
        "generated-catalog"
    }
}
pub struct Provider;
pub struct Producer;
impl WorthQueryApplicationProducerProvider<Catalog, Producer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "maintained-generated.provider.v1";
    fn operation_input(&self, source: &source::Row) -> operation::Generate {
        operation::Generate {
            identity: "generated-item".into(),
            directory: source.directory.clone(),
            name: source.note.clone(),
        }
    }
    fn idempotency_key(&self, _: &source::Row, source_identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(source_identity[..8].try_into().unwrap())
    }
}
impl WorthQueryApplicationProducerBinding<Catalog> for Producer {
    type Operation = operation::Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = operation::ItemOutput;
    const IDENTITY: &'static str = "maintained-generated.producer.v1";
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
    fn source_intent(&self) -> source::ReadDirectory {
        source::ReadDirectory {
            directory: self.directory.clone(),
        }
    }
}
