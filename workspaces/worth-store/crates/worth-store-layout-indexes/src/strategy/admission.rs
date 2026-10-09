use super::admitted_strategy::AdmittedLayoutStrategy;
use super::counter_planning::{
    declared_strategy_counter_envelope, family_requires_shape_specific_lookup_envelope,
};
use super::key_law_validation::admit_strategy_key_laws;
use super::{
    LayoutStrategyFamily, StrategyAuthorityBasis, StrategyDeclaration, StrategyDenial,
    StrategyInvariantSuite,
};
use crate::catalog::ArtifactFamilyLifecycleAdmission;
use crate::keyspace::{PhysicalKeyDomain, PhysicalKeyDomainWitness};

fn declare_strategy_from_basis(
    authority_basis: StrategyAuthorityBasis,
    family: LayoutStrategyFamily,
) -> Result<StrategyDeclaration, StrategyDenial> {
    let lifecycle = authority_basis.lifecycle();
    let key_domain = authority_basis.key_domain();
    require_matching_family(lifecycle, key_domain)?;
    require_supported_key_domain(family, key_domain.domain())?;
    let key_laws = admit_strategy_key_laws(family, key_domain)?;
    let declaration = match family {
        LayoutStrategyFamily::BTreeRange => {
            StrategyDeclaration::btree_range(authority_basis, key_laws)
        }
        LayoutStrategyFamily::BaselineLsmWriteOptimized => {
            StrategyDeclaration::baseline_lsm_write_optimized(authority_basis, key_laws)
        }
        _ => return Err(StrategyDenial::UnsupportedFamily),
    };
    require_complete_declaration(declaration)?;
    Ok(declaration)
}

pub(crate) fn admit_strategy_from_basis(
    authority_basis: StrategyAuthorityBasis,
    family: LayoutStrategyFamily,
) -> Result<AdmittedLayoutStrategy, StrategyDenial> {
    let declaration = declare_strategy_from_basis(authority_basis, family)?;
    let invariants = StrategyInvariantSuite::declare(declaration).into_admitted()?;
    Ok(AdmittedLayoutStrategy::new(declaration, invariants))
}

fn require_matching_family(
    lifecycle: ArtifactFamilyLifecycleAdmission,
    key_domain: PhysicalKeyDomainWitness,
) -> Result<(), StrategyDenial> {
    if lifecycle.family_id() != key_domain.family_id() {
        return Err(StrategyDenial::FamilyDoesNotMatchKeyDomain);
    }
    Ok(())
}

fn require_supported_key_domain(
    family: LayoutStrategyFamily,
    domain: PhysicalKeyDomain,
) -> Result<(), StrategyDenial> {
    let supported = match family {
        LayoutStrategyFamily::BTreeRange => matches!(
            domain,
            PhysicalKeyDomain::PageAddressKey
                | PhysicalKeyDomain::SegmentAddressKey
                | PhysicalKeyDomain::ExtentAddressKey
                | PhysicalKeyDomain::PhysicalReferenceKey
        ),
        LayoutStrategyFamily::BaselineLsmWriteOptimized => matches!(
            domain,
            PhysicalKeyDomain::WalRecordKey | PhysicalKeyDomain::BlobIdentityKey
        ),
        _ => return Err(StrategyDenial::UnsupportedFamily),
    };
    if supported {
        Ok(())
    } else {
        Err(match family {
            LayoutStrategyFamily::BTreeRange => {
                StrategyDenial::PhysicalKeyDomainDoesNotSupportBTree
            }
            LayoutStrategyFamily::BaselineLsmWriteOptimized => {
                StrategyDenial::PhysicalKeyDomainDoesNotSupportBaselineLsm
            }
            _ => StrategyDenial::UnsupportedFamily,
        })
    }
}

fn require_complete_declaration(declaration: StrategyDeclaration) -> Result<(), StrategyDenial> {
    if !declaration
        .capability()
        .allows_lane(declaration.access_lane())
    {
        return Err(StrategyDenial::StrategyDoesNotSupportDeclaredAccessLane);
    }
    if !family_requires_shape_specific_lookup_envelope(declaration.family())
        && declared_strategy_counter_envelope(declaration.family()).is_none()
    {
        return Err(StrategyDenial::StrategyDoesNotDeclarePlannedCounterEnvelope);
    }
    Ok(())
}
