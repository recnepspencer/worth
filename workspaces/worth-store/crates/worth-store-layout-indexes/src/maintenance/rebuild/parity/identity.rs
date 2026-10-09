use crate::strategy::LayoutStrategyFamily;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedIndexIdentityParity {
    Exact,
    SourceArtifactDoesNotProveIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedIndexOrderingParity {
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedIndexCoverageParity {
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedIndexCostEnvelopeParity {
    DeclaredEnvelopeMatched,
    SourceArtifactDoesNotProveDeclaredEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedIndexCounterShapeParity {
    ExactDeterministicPhysicalShape,
    StrategyDoesNotClaimDeterministicPhysicalShape,
}

pub(crate) const fn declared_counter_shape_parity(
    family: LayoutStrategyFamily,
) -> DerivedIndexCounterShapeParity {
    match family {
        LayoutStrategyFamily::BTreeRange => {
            DerivedIndexCounterShapeParity::ExactDeterministicPhysicalShape
        }
        _ => DerivedIndexCounterShapeParity::StrategyDoesNotClaimDeterministicPhysicalShape,
    }
}
