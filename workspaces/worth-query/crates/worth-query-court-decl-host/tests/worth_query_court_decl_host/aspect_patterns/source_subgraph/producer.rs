use super::*;
pub const PROFILE: &str = "source-subgraph";
pub struct Family;
impl WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = graph_read::source::Binding;
    type Entity = Generated;
    const IDENTITY: &'static str = "source-subgraph.family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] =
        &[WorthQueryProducerApplicability::new(
            PROFILE,
            WorthQueryProducerLifecyclePosture::Initial,
        )];
    fn profile_kind(_: &GraphRow) -> &'static str {
        PROFILE
    }
}
#[derive(Clone)]
pub struct Demand;
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = Family;
    fn source_intent(&self) -> graph_read::source::Read {
        graph_read::source::Read("source".into())
    }
}
pub struct Producer;
impl WorthQueryApplicationProducerBinding<Schema> for Producer {
    type Operation = generation::Binding;
    type OutputFamily = Family;
    type Provider = Provider;
    type OutputRole = generation::Root;
    const IDENTITY: &'static str = "source-subgraph.producer.v1";
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] = Family::SUPPORTED;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const REUSE_POLICY: &'static str = "exact-source";
}
pub struct Provider(pub Hashes, pub world::Fault);
impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
    type DenialValue = edit::ProducerDenialBinding;
    const SEMANTIC_IDENTITY: &'static str = "source-subgraph.producer.v1";
    fn operation_input(&self, row: &GraphRow) -> generation::Input {
        let mut input = generation::Input(row.clone());
        if self.1 == world::Fault::CorruptInputMember {
            let Some(member) = input.0.members.first_mut() else {
                panic!("fixed fixture must supply a member to corrupt");
            };
            member.value = 999;
        }
        input
    }
    fn idempotency_key(&self, _: &GraphRow, identity: &[u8; 32]) -> u64 {
        // This is native source-epoch provenance, not portable content identity.
        self.0.lock().unwrap().push(*identity);
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
    fn domain_denial_reason(&self, denial: &edit::Denial) -> Option<&'static str> {
        Some(match denial {
            edit::Denial::InvalidSourceGraph => "InvalidSourceGraph",
            edit::Denial::SourceMismatch => "SourceMismatch",
        })
    }
}
pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<Producer>()?;
    contracts.conditional::<readiness::Readiness>()?;
    Ok(())
}
