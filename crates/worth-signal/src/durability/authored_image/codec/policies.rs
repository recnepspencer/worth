use super::{Denial, Read, Write};
use crate::data::comparator::{
    InstalledSignalComparatorIdentity, InstalledSignalComparatorRole, VersionComparatorPolicy,
};
use crate::data::node::{
    EvaluationCondition, InstalledSignalConditionIdentity, InstalledSignalConditionRole,
};
use crate::data::output_equivalence::{
    InstalledOutputEquivalenceIdentity, OutputEquivalencePolicy,
};

pub(super) fn write_condition(
    out: &mut Write<'_>,
    value: &EvaluationCondition,
) -> Result<(), Denial> {
    match value {
        EvaluationCondition::Always => out.byte(0),
        EvaluationCondition::AspectFilter(mask) => {
            out.byte(1)?;
            out.mask(*mask)
        }
        EvaluationCondition::DeltaThreshold(threshold) => {
            out.byte(2)?;
            out.number(threshold.to_bits())
        }
        EvaluationCondition::OnDemand => out.byte(3),
        EvaluationCondition::Temporal(condition) => {
            out.byte(4)?;
            super::temporal::write(out, condition)
        }
        EvaluationCondition::Custom(key) => {
            out.byte(5)?;
            out.text(key)
        }
        EvaluationCondition::Installed(identity) => {
            out.byte(6)?;
            out.byte(match identity.role() {
                InstalledSignalConditionRole::Predicate => 0,
                InstalledSignalConditionRole::TemporalWake => 1,
            })
        }
    }
}
pub(super) fn read_condition(read: &mut Read<'_>) -> Result<EvaluationCondition, Denial> {
    Ok(match read.byte()? {
        0 => EvaluationCondition::Always,
        1 => EvaluationCondition::AspectFilter(read.mask()?),
        2 => EvaluationCondition::DeltaThreshold(f64::from_bits(read.number()?)),
        3 => EvaluationCondition::OnDemand,
        4 => EvaluationCondition::Temporal(super::temporal::read(read)?),
        5 => EvaluationCondition::Custom(read.text()?),
        6 => {
            let role = match read.byte()? {
                0 => InstalledSignalConditionRole::Predicate,
                1 => InstalledSignalConditionRole::TemporalWake,
                _ => return Err(Denial::Malformed),
            };
            EvaluationCondition::Installed(InstalledSignalConditionIdentity::new(
                read.graph_instance,
                read.node,
                role,
            ))
        }
        _ => return Err(Denial::Malformed),
    })
}
pub(super) fn write_comparator(
    out: &mut Write<'_>,
    value: &VersionComparatorPolicy,
) -> Result<(), Denial> {
    match value {
        VersionComparatorPolicy::Exact => out.byte(0),
        VersionComparatorPolicy::Tolerance { epsilon } => {
            out.byte(1)?;
            out.number(*epsilon)
        }
        VersionComparatorPolicy::OutputIdentity => out.byte(2),
        VersionComparatorPolicy::Custom { key } => {
            out.byte(3)?;
            out.text(key)
        }
        VersionComparatorPolicy::Installed { identity } => {
            out.byte(4)?;
            out.byte(match identity.role() {
                InstalledSignalComparatorRole::DependencyVersion => 0,
                InstalledSignalComparatorRole::OutputEquivalence => 1,
                InstalledSignalComparatorRole::ArtifactReuse => 2,
            })
        }
    }
}
pub(super) fn read_comparator(read: &mut Read<'_>) -> Result<VersionComparatorPolicy, Denial> {
    Ok(match read.byte()? {
        0 => VersionComparatorPolicy::Exact,
        1 => VersionComparatorPolicy::Tolerance {
            epsilon: read.number()?,
        },
        2 => VersionComparatorPolicy::OutputIdentity,
        3 => VersionComparatorPolicy::Custom { key: read.text()? },
        4 => {
            let role = match read.byte()? {
                0 => InstalledSignalComparatorRole::DependencyVersion,
                1 => InstalledSignalComparatorRole::OutputEquivalence,
                2 => InstalledSignalComparatorRole::ArtifactReuse,
                _ => return Err(Denial::Malformed),
            };
            VersionComparatorPolicy::Installed {
                identity: InstalledSignalComparatorIdentity::new(
                    read.graph_instance,
                    read.node,
                    role,
                ),
            }
        }
        _ => return Err(Denial::Malformed),
    })
}
pub(super) fn write_output(
    out: &mut Write<'_>,
    value: &OutputEquivalencePolicy,
) -> Result<(), Denial> {
    match value {
        OutputEquivalencePolicy::ExactAspectVersion => out.byte(0),
        OutputEquivalencePolicy::AspectVersionTolerance { epsilon } => {
            out.byte(1)?;
            out.number(*epsilon)
        }
        OutputEquivalencePolicy::OutputIdentity => out.byte(2),
        OutputEquivalencePolicy::Custom { key } => {
            out.byte(3)?;
            out.text(key)
        }
        OutputEquivalencePolicy::Installed { identity: _ } => out.byte(4),
    }
}
pub(super) fn read_output(read: &mut Read<'_>) -> Result<OutputEquivalencePolicy, Denial> {
    Ok(match read.byte()? {
        0 => OutputEquivalencePolicy::ExactAspectVersion,
        1 => OutputEquivalencePolicy::AspectVersionTolerance {
            epsilon: read.number()?,
        },
        2 => OutputEquivalencePolicy::OutputIdentity,
        3 => OutputEquivalencePolicy::Custom { key: read.text()? },
        4 => OutputEquivalencePolicy::Installed {
            identity: InstalledOutputEquivalenceIdentity::from_installed_comparator(
                InstalledSignalComparatorIdentity::new(
                    read.graph_instance,
                    read.node,
                    InstalledSignalComparatorRole::OutputEquivalence,
                ),
            )
            .ok_or(Denial::Malformed)?,
        },
        _ => return Err(Denial::Malformed),
    })
}
