use super::{AdmittedLayoutStrategy, LayoutStrategyFamily, StrategyDenial};
use crate::keyspace::{
    declare_comparator_law, require_canonical_key_encoding, require_prefix_law,
    require_range_bound_law, CanonicalKeyEncoding, ComparatorLaw, PhysicalKeyDomainWitness,
    PrefixLawWitness, RangeBoundLawWitness,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DeclaredKeyLawPosture {
    encoding: CanonicalKeyEncoding,
    comparator: ComparatorLaw,
    prefix: Option<PrefixLawWitness>,
    range: Option<RangeBoundLawWitness>,
}

impl DeclaredKeyLawPosture {
    pub(super) const fn encoding(self) -> CanonicalKeyEncoding {
        self.encoding
    }
    pub(super) const fn comparator(self) -> ComparatorLaw {
        self.comparator
    }
    pub(super) const fn prefix(self) -> Option<PrefixLawWitness> {
        self.prefix
    }
    pub(super) const fn range(self) -> Option<RangeBoundLawWitness> {
        self.range
    }
}

pub(super) fn admit_strategy_key_laws(
    family: LayoutStrategyFamily,
    key_domain: PhysicalKeyDomainWitness,
) -> Result<DeclaredKeyLawPosture, StrategyDenial> {
    let encoding = require_canonical_key_encoding(key_domain);
    let comparator = declare_comparator_law(encoding);
    let prefix = require_prefix_law(encoding).ok();
    let range = require_range_bound_law(comparator).ok();
    match family {
        LayoutStrategyFamily::BTreeRange => Ok(DeclaredKeyLawPosture {
            encoding,
            comparator,
            prefix: Some(prefix.ok_or(StrategyDenial::RangeOrPrefixLawRequired)?),
            range: Some(range.ok_or(StrategyDenial::RangeOrPrefixLawRequired)?),
        }),
        LayoutStrategyFamily::BaselineLsmWriteOptimized => Ok(DeclaredKeyLawPosture {
            encoding,
            comparator,
            prefix: None,
            range: None,
        }),
        _ => Err(StrategyDenial::UnsupportedFamily),
    }
}

impl AdmittedLayoutStrategy {
    pub const fn canonical_key_encoding(&self) -> Option<CanonicalKeyEncoding> {
        self.declaration.canonical_key_encoding()
    }
    pub const fn comparator_law(&self) -> Option<ComparatorLaw> {
        self.declaration.comparator_law()
    }
    pub const fn prefix_law(&self) -> Option<PrefixLawWitness> {
        self.declaration.prefix_law()
    }
    pub const fn range_bound_law(&self) -> Option<RangeBoundLawWitness> {
        self.declaration.range_bound_law()
    }
}
