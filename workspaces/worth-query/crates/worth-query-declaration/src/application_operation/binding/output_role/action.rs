use super::super::output::{ApplicationMutationOutputPosture, ApplicationMutationOutputPostureSet};

mod sealed {
    pub trait Sealed {}
}

/// Output action: the role names a record the mutation keeps, neither
/// creating nor retiring it.
pub struct WorthQueryPreserveOutput;
/// Output action: the role names a record the mutation creates.
pub struct WorthQueryCreateOutput;
/// Output action: the role names a record the mutation retires.
pub struct WorthQueryRetireOutput;
/// Output action: the role names a successor created by splitting a retired
/// parent record with a canonical discriminator.
pub struct WorthQuerySplitOutput;
/// Output action: the role names a successor produced by merging multiple
/// parent records under a declared merge keeper rule.
pub struct WorthQueryMergeOutput;

/// Uniform lineage posture kind across entities and relations, extending the
/// core create/preserve/retire postures with many-to-many `Split` and `Merge`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApplicationMutationLineagePostureKind {
    Preserve,
    Create,
    Retire,
    Split,
    Merge,
}

impl ApplicationMutationLineagePostureKind {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Preserve => "preserve",
            Self::Create => "create",
            Self::Retire => "retire",
            Self::Split => "split",
            Self::Merge => "merge",
        }
    }

    pub fn from_canonical_name(name: &str) -> Option<Self> {
        match name {
            "preserve" => Some(Self::Preserve),
            "create" => Some(Self::Create),
            "retire" => Some(Self::Retire),
            "split" => Some(Self::Split),
            "merge" => Some(Self::Merge),
            _ => None,
        }
    }

    pub const fn from_output_posture(posture: ApplicationMutationOutputPosture) -> Self {
        match posture {
            ApplicationMutationOutputPosture::Preserve => Self::Preserve,
            ApplicationMutationOutputPosture::Create => Self::Create,
            ApplicationMutationOutputPosture::Retire => Self::Retire,
        }
    }

    pub const fn try_to_output_posture(self) -> Option<ApplicationMutationOutputPosture> {
        match self {
            Self::Preserve => Some(ApplicationMutationOutputPosture::Preserve),
            Self::Create => Some(ApplicationMutationOutputPosture::Create),
            Self::Retire => Some(ApplicationMutationOutputPosture::Retire),
            Self::Split | Self::Merge => None,
        }
    }
}

impl From<ApplicationMutationOutputPosture> for ApplicationMutationLineagePostureKind {
    fn from(posture: ApplicationMutationOutputPosture) -> Self {
        Self::from_output_posture(posture)
    }
}

/// Bitset of admitted [`ApplicationMutationLineagePostureKind`]s for one
/// entity or relation lineage output-role family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationLineagePostureSet(u8);

impl ApplicationMutationLineagePostureSet {
    pub const PRESERVE: Self = Self(1);
    pub const CREATE: Self = Self(2);
    pub const RETIRE: Self = Self(4);
    pub const SPLIT: Self = Self(8);
    pub const MERGE: Self = Self(16);
    pub const ALL: Self = Self(31);

    pub const fn one(posture: ApplicationMutationLineagePostureKind) -> Self {
        match posture {
            ApplicationMutationLineagePostureKind::Preserve => Self::PRESERVE,
            ApplicationMutationLineagePostureKind::Create => Self::CREATE,
            ApplicationMutationLineagePostureKind::Retire => Self::RETIRE,
            ApplicationMutationLineagePostureKind::Split => Self::SPLIT,
            ApplicationMutationLineagePostureKind::Merge => Self::MERGE,
        }
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn allows(self, posture: ApplicationMutationLineagePostureKind) -> bool {
        self.0 & Self::one(posture).0 != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits != 0 && bits & !Self::ALL.0 == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub const fn from_output_posture_set(set: ApplicationMutationOutputPostureSet) -> Self {
        Self(set.bits())
    }
}

/// The action an output role performs on its record: implemented only by
/// [`WorthQueryPreserveOutput`], [`WorthQueryCreateOutput`] and
/// [`WorthQueryRetireOutput`], and sealed against other implementations.
pub trait WorthQueryApplicationOutputAction: sealed::Sealed + 'static {
    const POSTURE: ApplicationMutationOutputPosture;
}

/// The lineage action an entity or relation output role performs on its
/// record: implemented only by [`WorthQueryPreserveOutput`],
/// [`WorthQueryCreateOutput`], [`WorthQueryRetireOutput`],
/// [`WorthQuerySplitOutput`], and [`WorthQueryMergeOutput`].
///
/// Sealed so external callers cannot forge a sixth lineage posture action:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_operation::{
///     ApplicationMutationLineagePostureKind, WorthQueryApplicationLineageOutputAction,
/// };
///
/// struct ForgedAction;
/// impl WorthQueryApplicationLineageOutputAction for ForgedAction {
///     const LINEAGE_POSTURE: ApplicationMutationLineagePostureKind =
///         ApplicationMutationLineagePostureKind::Split;
/// }
/// ```
pub trait WorthQueryApplicationLineageOutputAction: sealed::Sealed + 'static {
    const LINEAGE_POSTURE: ApplicationMutationLineagePostureKind;
}

impl sealed::Sealed for WorthQueryPreserveOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryPreserveOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Preserve;
}
impl WorthQueryApplicationLineageOutputAction for WorthQueryPreserveOutput {
    const LINEAGE_POSTURE: ApplicationMutationLineagePostureKind =
        ApplicationMutationLineagePostureKind::Preserve;
}

impl sealed::Sealed for WorthQueryCreateOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryCreateOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Create;
}
impl WorthQueryApplicationLineageOutputAction for WorthQueryCreateOutput {
    const LINEAGE_POSTURE: ApplicationMutationLineagePostureKind =
        ApplicationMutationLineagePostureKind::Create;
}

impl sealed::Sealed for WorthQueryRetireOutput {}
impl WorthQueryApplicationOutputAction for WorthQueryRetireOutput {
    const POSTURE: ApplicationMutationOutputPosture = ApplicationMutationOutputPosture::Retire;
}
impl WorthQueryApplicationLineageOutputAction for WorthQueryRetireOutput {
    const LINEAGE_POSTURE: ApplicationMutationLineagePostureKind =
        ApplicationMutationLineagePostureKind::Retire;
}

impl sealed::Sealed for WorthQuerySplitOutput {}
impl WorthQueryApplicationLineageOutputAction for WorthQuerySplitOutput {
    const LINEAGE_POSTURE: ApplicationMutationLineagePostureKind =
        ApplicationMutationLineagePostureKind::Split;
}

impl sealed::Sealed for WorthQueryMergeOutput {}
impl WorthQueryApplicationLineageOutputAction for WorthQueryMergeOutput {
    const LINEAGE_POSTURE: ApplicationMutationLineagePostureKind =
        ApplicationMutationLineagePostureKind::Merge;
}
