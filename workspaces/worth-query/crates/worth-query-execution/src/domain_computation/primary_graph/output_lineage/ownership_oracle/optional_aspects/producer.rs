use super::*;
pub(super) struct Family;
const INITIAL: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "optional-birth",
    WorthQueryProducerLifecyclePosture::Initial,
);
const KEEP: WorthQueryProducerApplicability = WorthQueryProducerApplicability::new(
    "optional-birth",
    WorthQueryProducerLifecyclePosture::Preserve,
);
impl<Schema: BirthSchema> WorthQueryProducerOutputFamily<Schema> for Family {
    type Source = PlanarReadBinding<Schema>;
    type Entity = PresentNode;
    const IDENTITY: &'static str = "worth.query.test.optional-aspect-family.v1";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[INITIAL, KEEP];
    fn profile_kind(_: &PlanarReadResult) -> &'static str {
        "optional-birth"
    }
}
pub(super) struct Initial<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct Preserve<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct Provider {
    pub(super) incomplete: bool,
}
macro_rules! producer {
    ($producer:ident, $keep:literal, $role:ident, $posture:ident, $identity:literal, $input:ident) => {
        impl<Schema: BirthSchema> WorthQueryApplicationProducerBinding<Schema>
            for $producer<Schema>
        {
            type Operation = Binding<Schema, $keep>;
            type OutputFamily = Family;
            type Provider = Provider;
            type OutputRole = $role<Schema>;
            const IDENTITY: &'static str = $identity;
            const APPLICABILITY: &'static [WorthQueryProducerApplicability] = &[$posture];
            const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
            const RESOURCE_POLICY: &'static str = "bounded-synchronous";
            const REUSE_POLICY: &'static str = "exact-source";
        }
        impl<Schema: BirthSchema> WorthQueryApplicationProducerProvider<Schema, $producer<Schema>>
            for Provider
        {
            const SEMANTIC_IDENTITY: &'static str = $identity;
            fn operation_input(&self, source: &PlanarReadResult) -> $input {
                $input {
                    scope_key: source.body_key.clone(),
                    incomplete: self.incomplete,
                }
            }
            fn idempotency_key(&self, _: &PlanarReadResult, identity: &[u8; 32]) -> u64 {
                u64::from_be_bytes(identity[..8].try_into().unwrap())
            }
            fn demand_resources(&self, _: &PlanarReadResult) -> WorthQueryProducerDemandResources {
                planar_producer_resources()
            }
        }
    };
}
producer!(
    Initial,
    false,
    BornPresent,
    INITIAL,
    "worth.query.test.optional-initial.v1",
    Input
);
producer!(
    Preserve,
    true,
    KeptPresent,
    KEEP,
    "worth.query.test.optional-preserve.v1",
    KeepInput
);
