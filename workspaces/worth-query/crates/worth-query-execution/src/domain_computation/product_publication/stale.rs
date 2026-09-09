use worth_runtime_world::facade::{
    NoEffectCause, NoEffectCompositePublication, ProductBranchObservation,
    ProductBranchReferenceSnapshot,
};

/// A selected product occurrence lost currentness before any owner effect.
/// Retains World's exact comparison evidence without inventing changed facts.
#[derive(Debug)]
pub struct WorthQueryProductStaleApplication {
    publication: NoEffectCompositePublication,
}

impl WorthQueryProductStaleApplication {
    pub(in crate::domain_computation) fn new(publication: NoEffectCompositePublication) -> Self {
        assert_eq!(publication.cause(), NoEffectCause::StaleExpectedProductHead);
        assert!(publication.expected_head().is_some());
        Self { publication }
    }

    pub fn expected_product(&self) -> &ProductBranchObservation {
        self.publication
            .expected_head()
            .expect("World stale retains its expected product")
    }

    pub fn observed_product(&self) -> Option<&ProductBranchReferenceSnapshot> {
        self.publication.observed_head()
    }
}
