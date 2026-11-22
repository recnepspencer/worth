//! Typed Phase 2 lineage output postures (`Split` and `Merge` alongside
//! `Create`, `Preserve`, and `Retire`) and portable erased lowering.

use std::collections::BTreeSet;
use std::fmt;

use crate::application_naming::{ApplicationNameDiscriminator, WorthQueryName};

use super::action::ApplicationMutationLineagePostureKind;

/// Subject class governed by a lineage output role or family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApplicationMutationLineageSubjectKind {
    Entity,
    Relation,
}

impl ApplicationMutationLineageSubjectKind {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Relation => "relation",
        }
    }

    pub fn from_canonical_name(name: &str) -> Option<Self> {
        match name {
            "entity" => Some(Self::Entity),
            "relation" => Some(Self::Relation),
            _ => None,
        }
    }
}

/// Keeper rule for a [`WorthQueryLineageOutputPosture::Merge`] output.
///
/// Cross-kind keepers are rejected at compile time:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
/// use worth_query_declaration::facade::application_operation::WorthQueryMergeKeeper;
///
/// struct Face;
/// struct Edge;
///
/// fn misuse(edge: WorthQueryName<Edge>) -> WorthQueryMergeKeeper<Face> {
///     WorthQueryMergeKeeper::KeepParent(edge)
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryMergeKeeper<K, D = ()> {
    /// Preserves one listed parent's persistent name; must be a member of the
    /// merge's `parents`.
    KeepParent(WorthQueryName<K>),
    /// Mints a distinct persistent name carrying all sorted parent identities
    /// and the supplied typed discriminator.
    MintNew { discriminator: D },
}

/// Typed lineage posture declared for an entity or relation output of kind `K`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryLineageOutputPosture<K, D = ()> {
    Create,
    Preserve,
    Retire,
    Split {
        parent: WorthQueryName<K>,
        discriminator: D,
    },
    Merge {
        parents: Vec<WorthQueryName<K>>,
        keeper: WorthQueryMergeKeeper<K, D>,
    },
}

/// Structural refusal when lowering a [`WorthQueryLineageOutputPosture`] into
/// its portable canonical form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryLineagePostureDenial {
    EmptyMergeParents,
    InsufficientMergeParents { count: usize },
    DuplicateMergeParent { parent_hex: String },
    InvalidMergeKeeper { keeper_hex: String },
}

impl fmt::Display for WorthQueryLineagePostureDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMergeParents => {
                formatter.write_str("merge posture requires at least two parent names")
            }
            Self::InsufficientMergeParents { count } => write!(
                formatter,
                "merge posture requires at least two parent names, received {count}"
            ),
            Self::DuplicateMergeParent { parent_hex } => {
                write!(
                    formatter,
                    "merge posture lists duplicate parent `{parent_hex}`"
                )
            }
            Self::InvalidMergeKeeper { keeper_hex } => write!(
                formatter,
                "merge keeper `{keeper_hex}` is not a member of the declared merge parents"
            ),
        }
    }
}

impl std::error::Error for WorthQueryLineagePostureDenial {}

/// Portable, type-erased merge keeper lowered from [`WorthQueryMergeKeeper`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErasedMergeKeeper {
    KeepParent([u8; 32]),
    MintNew {
        canonical_discriminator: Vec<u8>,
        referenced_name_digests: Vec<[u8; 32]>,
    },
}

/// Portable, type-erased lineage posture lowered from
/// [`WorthQueryLineageOutputPosture`] for Relational validation and emission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErasedLineageOutputPosture {
    Create,
    Preserve,
    Retire,
    Split {
        parent_digest: [u8; 32],
        canonical_discriminator: Vec<u8>,
        referenced_name_digests: Vec<[u8; 32]>,
    },
    Merge {
        parent_digests: Vec<[u8; 32]>,
        keeper: ErasedMergeKeeper,
    },
}

impl ErasedLineageOutputPosture {
    pub const fn kind(&self) -> ApplicationMutationLineagePostureKind {
        match self {
            Self::Create => ApplicationMutationLineagePostureKind::Create,
            Self::Preserve => ApplicationMutationLineagePostureKind::Preserve,
            Self::Retire => ApplicationMutationLineagePostureKind::Retire,
            Self::Split { .. } => ApplicationMutationLineagePostureKind::Split,
            Self::Merge { .. } => ApplicationMutationLineagePostureKind::Merge,
        }
    }
}

impl<K, D> WorthQueryLineageOutputPosture<K, D> {
    pub const fn kind(&self) -> ApplicationMutationLineagePostureKind {
        match self {
            Self::Create => ApplicationMutationLineagePostureKind::Create,
            Self::Preserve => ApplicationMutationLineagePostureKind::Preserve,
            Self::Retire => ApplicationMutationLineagePostureKind::Retire,
            Self::Split { .. } => ApplicationMutationLineagePostureKind::Split,
            Self::Merge { .. } => ApplicationMutationLineagePostureKind::Merge,
        }
    }

    /// Checks structural posture invariants (merge keeper membership, parent
    /// uniqueness, and minimum parent cardinality) before commit.
    pub fn validate_shape(&self) -> Result<(), WorthQueryLineagePostureDenial> {
        let Self::Merge { parents, keeper } = self else {
            return Ok(());
        };
        if parents.is_empty() {
            return Err(WorthQueryLineagePostureDenial::EmptyMergeParents);
        }
        if let WorthQueryMergeKeeper::KeepParent(keeper_name) = keeper
            && !parents.contains(keeper_name)
        {
            return Err(WorthQueryLineagePostureDenial::InvalidMergeKeeper {
                keeper_hex: keeper_name.render_hex(),
            });
        }
        if parents.len() < 2 {
            return Err(WorthQueryLineagePostureDenial::InsufficientMergeParents {
                count: parents.len(),
            });
        }
        let mut seen = BTreeSet::new();
        for parent in parents {
            if !seen.insert(*parent.as_bytes()) {
                return Err(WorthQueryLineagePostureDenial::DuplicateMergeParent {
                    parent_hex: parent.render_hex(),
                });
            }
        }
        Ok(())
    }

    /// Validates posture shape and encodes any typed discriminator into its
    /// canonical domain-tagged byte stream plus embedded name digests.
    pub fn to_erased_posture(
        &self,
    ) -> Result<ErasedLineageOutputPosture, WorthQueryLineagePostureDenial>
    where
        D: ApplicationNameDiscriminator,
    {
        self.validate_shape()?;
        match self {
            Self::Create => Ok(ErasedLineageOutputPosture::Create),
            Self::Preserve => Ok(ErasedLineageOutputPosture::Preserve),
            Self::Retire => Ok(ErasedLineageOutputPosture::Retire),
            Self::Split {
                parent,
                discriminator,
            } => {
                let (canonical_discriminator, referenced_name_digests) =
                    discriminator.canonical_bytes_with_references();
                Ok(ErasedLineageOutputPosture::Split {
                    parent_digest: *parent.as_bytes(),
                    canonical_discriminator,
                    referenced_name_digests,
                })
            }
            Self::Merge { parents, keeper } => {
                let mut parent_digests: Vec<[u8; 32]> =
                    parents.iter().map(|parent| *parent.as_bytes()).collect();
                parent_digests.sort_unstable();
                let erased_keeper = match keeper {
                    WorthQueryMergeKeeper::KeepParent(name) => {
                        ErasedMergeKeeper::KeepParent(*name.as_bytes())
                    }
                    WorthQueryMergeKeeper::MintNew { discriminator } => {
                        let (canonical_discriminator, referenced_name_digests) =
                            discriminator.canonical_bytes_with_references();
                        ErasedMergeKeeper::MintNew {
                            canonical_discriminator,
                            referenced_name_digests,
                        }
                    }
                };
                Ok(ErasedLineageOutputPosture::Merge {
                    parent_digests,
                    keeper: erased_keeper,
                })
            }
        }
    }
}
