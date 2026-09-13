mod blast_radius;
mod cause;
mod damaged_range;
mod format_field;

pub use blast_radius::PhysicalBlastRadius;
pub use cause::PhysicalDamageCause;
pub use damaged_range::{PhysicalByteRange, PhysicalByteRangeDenial};
pub use format_field::PhysicalFormatField;

use crate::validation::PhysicalArtifactScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalDamageLocalization {
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
}

impl PhysicalDamageLocalization {
    pub const fn new(
        scope: PhysicalArtifactScope,
        cause: PhysicalDamageCause,
        damaged_range: PhysicalByteRange,
        field: Option<PhysicalFormatField>,
        blast_radius: PhysicalBlastRadius,
    ) -> Self {
        Self {
            scope,
            cause,
            damaged_range,
            field,
            blast_radius,
        }
    }

    pub const fn scope(self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn cause(self) -> PhysicalDamageCause {
        self.cause
    }

    pub const fn damaged_range(self) -> PhysicalByteRange {
        self.damaged_range
    }

    pub const fn field(self) -> Option<PhysicalFormatField> {
        self.field
    }

    pub const fn blast_radius(self) -> PhysicalBlastRadius {
        self.blast_radius
    }
}
