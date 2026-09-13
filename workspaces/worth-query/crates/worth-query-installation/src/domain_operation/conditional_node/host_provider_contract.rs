use worth_foundational::facade::{
    AspectMask, AspectValue, ContractValidatedAspectArtifact, ContractValidatedAspectValueView,
    FieldKey, ProjectionMask,
};

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryConditionalProjectedValue<'a> {
    artifact: &'a ContractValidatedAspectArtifact,
    mask: &'a AspectMask<ProjectionMask>,
}

impl<'a> WorthQueryConditionalProjectedValue<'a> {
    #[doc(hidden)]
    pub fn from_runtime_projection(
        artifact: &'a ContractValidatedAspectArtifact,
        mask: &'a AspectMask<ProjectionMask>,
    ) -> Self {
        Self { artifact, mask }
    }

    pub fn scalar(self) -> Option<&'a AspectValue> {
        if !self.mask.is_whole_aspect() {
            return None;
        }
        match self.artifact.payload().view() {
            ContractValidatedAspectValueView::Scalar(value) => Some(value),
            ContractValidatedAspectValueView::Struct(_) => None,
        }
    }

    pub fn field(self, field: &FieldKey) -> Option<&'a AspectValue> {
        let admitted = self.mask.is_whole_aspect()
            || self
                .mask
                .paths()
                .iter()
                .any(|path| path.fields().len() == 1 && path.fields().first() == Some(field));
        if !admitted {
            return None;
        }
        match self.artifact.payload().view() {
            ContractValidatedAspectValueView::Struct(value) => value.get(field),
            ContractValidatedAspectValueView::Scalar(_) => None,
        }
    }
}

/// One declared dependency value visible to a host conditional provider.
///
/// Absence is explicit so a provider cannot confuse an unavailable dependency
/// with a present value whose payload happens to be empty.
#[derive(Clone, Copy, Debug)]
pub enum WorthQueryConditionalObservedValue<'a> {
    Present(WorthQueryConditionalProjectedValue<'a>),
    Absent,
}

/// Descriptive truth basis shared by every dependency in one evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryConditionalObservationTruthBasis<'a> {
    branch_identity: &'a str,
    snapshot_identity: &'a str,
}

impl<'a> WorthQueryConditionalObservationTruthBasis<'a> {
    #[doc(hidden)]
    pub fn from_runtime_truth(branch_identity: &'a str, snapshot_identity: &'a str) -> Self {
        Self {
            branch_identity,
            snapshot_identity,
        }
    }

    pub fn branch_identity(self) -> &'a str {
        self.branch_identity
    }

    pub fn snapshot_identity(self) -> &'a str {
        self.snapshot_identity
    }
}

/// Previous and current values for one dependency declaration ordinal.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryConditionalDependencyObservation<'a> {
    declaration_ordinal: usize,
    previous: WorthQueryConditionalObservedValue<'a>,
    current: WorthQueryConditionalObservedValue<'a>,
}

impl<'a> WorthQueryConditionalDependencyObservation<'a> {
    #[doc(hidden)]
    pub fn from_runtime_observation(
        declaration_ordinal: usize,
        previous: WorthQueryConditionalObservedValue<'a>,
        current: WorthQueryConditionalObservedValue<'a>,
    ) -> Self {
        Self {
            declaration_ordinal,
            previous,
            current,
        }
    }

    pub fn declaration_ordinal(self) -> usize {
        self.declaration_ordinal
    }

    pub fn previous(self) -> WorthQueryConditionalObservedValue<'a> {
        self.previous
    }

    pub fn current(self) -> WorthQueryConditionalObservedValue<'a> {
        self.current
    }
}

/// Immutable, dependency-indexed view supplied to one admitted host provider.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryConditionalObservationView<'a> {
    basis: WorthQueryConditionalObservationTruthBasis<'a>,
    dependencies: &'a [WorthQueryConditionalDependencyObservation<'a>],
}

impl<'a> WorthQueryConditionalObservationView<'a> {
    #[doc(hidden)]
    pub fn from_runtime_observations(
        basis: WorthQueryConditionalObservationTruthBasis<'a>,
        dependencies: &'a [WorthQueryConditionalDependencyObservation<'a>],
    ) -> Self {
        Self {
            basis,
            dependencies,
        }
    }

    pub fn basis(self) -> WorthQueryConditionalObservationTruthBasis<'a> {
        self.basis
    }

    pub fn dependencies(self) -> &'a [WorthQueryConditionalDependencyObservation<'a>] {
        self.dependencies
    }

    pub fn dependency(
        self,
        declaration_ordinal: usize,
    ) -> Option<WorthQueryConditionalDependencyObservation<'a>> {
        self.dependencies
            .iter()
            .copied()
            .find(|observation| observation.declaration_ordinal == declaration_ordinal)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHostPredicateDecision {
    Satisfied,
    Unsatisfied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHostPredicateFailureKind {
    ObservationUnsupported,
    ProviderUnavailable,
    ProviderFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryHostPredicateFailure {
    kind: WorthQueryHostPredicateFailureKind,
    detail: String,
}

impl WorthQueryHostPredicateFailure {
    pub fn new(kind: WorthQueryHostPredicateFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryHostPredicateFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Host implementation of one typed conditional-node predicate.
///
/// The provider returns domain truth only. Query adapts that truth into the
/// installed Runtime Bridge contract; neither raw Signal eligibility nor wake
/// authority can cross this boundary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryHostProviderHeapRetention(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorthQueryHostProviderRetentionOverflow;

impl WorthQueryHostProviderHeapRetention {
    pub const fn none() -> Self {
        Self(0)
    }

    pub fn try_from_parts(
        parts: impl IntoIterator<Item = u64>,
    ) -> Result<Self, WorthQueryHostProviderRetentionOverflow> {
        parts
            .into_iter()
            .try_fold(0u64, |total, part| {
                total
                    .checked_add(part)
                    .ok_or(WorthQueryHostProviderRetentionOverflow)
            })
            .map(Self)
    }

    pub const fn bytes(self) -> u64 {
        self.0
    }

    pub fn arc_allocation_bytes<T: ?Sized>(value: &T) -> u64 {
        let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::for_value(value))
            .expect("one live value has a representable Arc allocation layout");
        layout.pad_to_align().size() as u64
    }
}

pub trait WorthQueryHostConditionalPredicateProvider<Node>: Send + Sync + 'static {
    const SEMANTIC_IDENTITY: &'static str;

    fn retained_heap_bytes(
        &self,
    ) -> Result<WorthQueryHostProviderHeapRetention, WorthQueryHostProviderRetentionOverflow>;

    fn evaluate(
        &self,
        observation: WorthQueryConditionalObservationView<'_>,
    ) -> Result<WorthQueryHostPredicateDecision, WorthQueryHostPredicateFailure>;
}

/// Host-owned semantic output comparison for an installed conditional node.
/// The values are node-local semantic output versions, never raw dependency
/// versions or cross-runtime authority.
pub trait WorthQueryHostConditionalOutputComparatorProvider<Node>: Send + Sync + 'static {
    fn semantic_identity(&self) -> &'static str;

    fn retained_heap_bytes(
        &self,
    ) -> Result<WorthQueryHostProviderHeapRetention, WorthQueryHostProviderRetentionOverflow>;

    fn has_meaningful_change(
        &self,
        cached: u64,
        current: u64,
    ) -> Result<bool, WorthQueryHostPredicateFailure>;
}

/// Host-owned computation of one conditional node's semantic output version.
/// Query supplies the monotonic attempt only as a fallback; domain providers
/// may project a stronger semantic version from their installed live state.
pub trait WorthQueryHostConditionalOutputVersionProvider<Node>: Send + Sync + 'static {
    fn semantic_identity(&self) -> &'static str;

    fn retained_heap_bytes(
        &self,
    ) -> Result<WorthQueryHostProviderHeapRetention, WorthQueryHostProviderRetentionOverflow>;

    fn output_version(&self, fallback_attempt: u64) -> Result<u64, WorthQueryHostPredicateFailure>;
}
