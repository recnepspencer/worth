use std::fmt;

use super::{validate_discriminator_identity, ApplicationNameDiscriminator, ErasedWorthQueryName};

/// How a held persistent name chooses among its live descendants.
///
/// Every selection first finds the held name's live descendants at the
/// reader's basis (the held name itself when it is still live). When none is
/// live, every selection resolves to `Retired { at_commit }`. Otherwise:
///
/// - [`NameSelection::Exact`]: one live descendant resolves `Exact`; two or
///   more refuse `Ambiguous { candidates }`. It never picks a least member.
/// - [`NameSelection::AllDescendants`]: one resolves `Exact`; two or more
///   resolve `Plural` in canonical name order.
/// - [`NameSelection::BoundedByCarriers`]: keeps the live descendants whose
///   declared carriers include every requested carrier; at least one carrier
///   is required, so the filter is never a wildcard.
/// - [`NameSelection::KeepSide`]: keeps the live descendants born with exactly
///   this canonical split discriminator.
///
/// For the two filters, one match resolves `Exact`, two or more refuse
/// `Ambiguous`, and zero refuse `NoMatchingCandidate { live_candidates }`; a
/// filter never reports a live name as retired.
///
/// Filters are built only through their typed constructors. Raw bytes cannot
/// become a side discriminator:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::NameSelection;
///
/// let _forged = NameSelection::KeepSide { discriminator: vec![1u8, 2, 3] };
/// ```
///
/// Nor can an unchecked carrier list:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::NameSelection;
///
/// let _forged = NameSelection::BoundedByCarriers { carriers: Vec::new() };
/// ```
///
/// The typed constructors canonicalize their input:
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder, NameSelection,
///     NameSelectionDenial, WorthQueryName,
/// };
///
/// struct Face;
/// struct Side(u8);
///
/// impl ApplicationNameDiscriminator for Side {
///     const DISCRIMINATOR_IDENTITY: &'static str = "example.side.v1";
///     fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
///         encoder.field("side", &self.0);
///     }
/// }
///
/// fn select(a: WorthQueryName<Face>, b: WorthQueryName<Face>) {
///     let forward = NameSelection::bounded_by_carriers([a.erased(), b.erased(), a.erased()]).unwrap();
///     let backward = NameSelection::bounded_by_carriers([b.erased(), a.erased()]).unwrap();
///     assert_eq!(forward, backward);
/// }
///
/// assert_eq!(
///     NameSelection::bounded_by_carriers([]),
///     Err(NameSelectionDenial::EmptyCarrierSet)
/// );
///
/// let side = NameSelection::keep_side(&Side(1)).unwrap();
/// let NameSelection::KeepSide { discriminator } = &side else { unreachable!() };
/// assert_eq!(discriminator.as_bytes(), Side(1).canonical_bytes().as_slice());
/// let _check: fn(WorthQueryName<Face>, WorthQueryName<Face>) = select;
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NameSelection {
    Exact,
    AllDescendants,
    BoundedByCarriers {
        carriers: NameCarrierSet,
    },
    KeepSide {
        discriminator: NameSideDiscriminator,
    },
}

impl NameSelection {
    /// Selects the live descendants bounded by every listed carrier. The
    /// carriers are sorted and deduplicated, so argument order never matters.
    /// Refuses an empty carrier list, which would otherwise match everything.
    pub fn bounded_by_carriers(
        carriers: impl IntoIterator<Item = ErasedWorthQueryName>,
    ) -> Result<Self, NameSelectionDenial> {
        let mut carriers: Vec<ErasedWorthQueryName> = carriers.into_iter().collect();
        if carriers.is_empty() {
            return Err(NameSelectionDenial::EmptyCarrierSet);
        }
        carriers.sort_unstable();
        carriers.dedup();
        Ok(Self::BoundedByCarriers {
            carriers: NameCarrierSet { carriers },
        })
    }

    /// Selects the live descendant born on `side`, encoded by the canonical
    /// [`super::ApplicationNameDiscriminatorEncoder`]. Refuses a discriminator
    /// type whose declared identity is malformed.
    pub fn keep_side<D: ApplicationNameDiscriminator>(
        side: &D,
    ) -> Result<Self, NameSelectionDenial> {
        validate_discriminator_identity("", D::DISCRIMINATOR_IDENTITY).map_err(|_| {
            NameSelectionDenial::InvalidDiscriminatorIdentity {
                discriminator_identity: D::DISCRIMINATOR_IDENTITY.to_owned(),
            }
        })?;
        Ok(Self::KeepSide {
            discriminator: NameSideDiscriminator {
                canonical: side.canonical_bytes(),
            },
        })
    }
}

/// Sorted, deduplicated, nonempty carriers of a
/// [`NameSelection::BoundedByCarriers`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NameCarrierSet {
    carriers: Vec<ErasedWorthQueryName>,
}

impl NameCarrierSet {
    /// Carriers in canonical (ascending digest) order without duplicates.
    pub fn as_slice(&self) -> &[ErasedWorthQueryName] {
        &self.carriers
    }
}

/// Canonical discriminator bytes of a [`NameSelection::KeepSide`], produced
/// only by [`NameSelection::keep_side`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NameSideDiscriminator {
    canonical: Vec<u8>,
}

impl NameSideDiscriminator {
    /// The domain-tagged canonical discriminator bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.canonical
    }
}

/// Typed refusal while building a [`NameSelection`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameSelectionDenial {
    /// The side discriminator type declares a malformed identity string.
    InvalidDiscriminatorIdentity { discriminator_identity: String },
    /// A carrier filter names no carrier.
    EmptyCarrierSet,
}

impl fmt::Display for NameSelectionDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDiscriminatorIdentity {
                discriminator_identity,
            } => write!(
                formatter,
                "side discriminator declares malformed identity `{discriminator_identity}`"
            ),
            Self::EmptyCarrierSet => formatter.write_str("carrier filter names no carrier"),
        }
    }
}

impl std::error::Error for NameSelectionDenial {}
