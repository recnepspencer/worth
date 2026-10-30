use worth_runtime_bridge::facade::{
    BridgeTemporalAsyncCertificationAsyncLifecycleSection,
    BridgeTemporalAsyncCertificationBasisSection,
    BridgeTemporalAsyncCertificationBundleSealed,
    BridgeTemporalAsyncCertificationCounters,
    BridgeTemporalAsyncCertificationDiagnosticsRichness,
    BridgeTemporalAsyncCertificationFailureSection,
    BridgeTemporalAsyncCertificationMixedCauseSection,
    BridgeTemporalAsyncCertificationResumeSection,
};

fn fake<T>() -> T {
    panic!("fixture should never run")
}

fn main() {
    let _ = BridgeTemporalAsyncCertificationBundleSealed {
        schema_version: fake(),
        active_subscription_identity: fake(),
        admitted_subscription_identity: fake(),
        diagnostics_richness: BridgeTemporalAsyncCertificationDiagnosticsRichness::Minimal,
        basis_section: fake::<BridgeTemporalAsyncCertificationBasisSection>(),
        async_section: fake::<BridgeTemporalAsyncCertificationAsyncLifecycleSection>(),
        mixed_cause_section: fake::<BridgeTemporalAsyncCertificationMixedCauseSection>(),
        resume_section: fake::<BridgeTemporalAsyncCertificationResumeSection>(),
        failure_section: fake::<BridgeTemporalAsyncCertificationFailureSection>(),
        counters: fake::<BridgeTemporalAsyncCertificationCounters>(),
        semantic_digest: fake(),
        digest: fake(),
    };
}
