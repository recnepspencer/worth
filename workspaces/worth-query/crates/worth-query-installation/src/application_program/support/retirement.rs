use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

/// Exact execution-owned leases that still require one installed program.
///
/// Installation defines the closure contract but does not discover these
/// users. Execution fills the inventory from its branch, retained-read, and
/// recovery owners before support may leave ordinary service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramSupportRetirementInventory {
    revision: ApplicationProgramRevision,
    current_branches: usize,
    retained_interpretations: usize,
    mandatory_custody: usize,
    retained_program_bytes: usize,
}

/// Support users known when a live-branch inventory cannot be completed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramSupportPartialRetirementInventory {
    revision: ApplicationProgramRevision,
    retained_interpretations: usize,
    mandatory_custody: usize,
    retained_program_bytes: usize,
}

impl WorthQueryProgramSupportPartialRetirementInventory {
    pub fn inspected(
        revision: ApplicationProgramRevision,
        retained_interpretations: usize,
        mandatory_custody: usize,
        retained_program_bytes: usize,
    ) -> Self {
        Self {
            revision,
            retained_interpretations,
            mandatory_custody,
            retained_program_bytes,
        }
    }

    pub fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
    }

    pub const fn retained_interpretations(&self) -> usize {
        self.retained_interpretations
    }

    pub const fn mandatory_custody(&self) -> usize {
        self.mandatory_custody
    }

    pub const fn retained_program_bytes(&self) -> usize {
        self.retained_program_bytes
    }
}

impl WorthQueryProgramSupportRetirementInventory {
    pub fn inspected(
        revision: ApplicationProgramRevision,
        current_branches: usize,
        retained_interpretations: usize,
        mandatory_custody: usize,
        retained_program_bytes: usize,
    ) -> Self {
        Self {
            revision,
            current_branches,
            retained_interpretations,
            mandatory_custody,
            retained_program_bytes,
        }
    }

    pub fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
    }

    pub const fn current_branches(&self) -> usize {
        self.current_branches
    }

    pub const fn retained_interpretations(&self) -> usize {
        self.retained_interpretations
    }

    pub const fn mandatory_custody(&self) -> usize {
        self.mandatory_custody
    }

    pub const fn retained_program_bytes(&self) -> usize {
        self.retained_program_bytes
    }

    pub const fn can_retire(&self) -> bool {
        self.current_branches == 0
            && self.retained_interpretations == 0
            && self.mandatory_custody == 0
    }
}

/// Why one installed program cannot leave ordinary service yet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramSupportRetirementDenial {
    UnrosteredProgram {
        revision: ApplicationProgramRevision,
    },
    AlreadyRetired {
        revision: ApplicationProgramRevision,
    },
    RetirementInProgress {
        revision: ApplicationProgramRevision,
    },
    CurrentBranches(WorthQueryProgramSupportRetirementInventory),
    RetainedInterpretation(WorthQueryProgramSupportRetirementInventory),
    MandatoryCustody(WorthQueryProgramSupportRetirementInventory),
    InventoryUnavailable(WorthQueryProgramSupportPartialRetirementInventory),
}

impl WorthQueryProgramSupportRetirementDenial {
    pub const fn inventory(&self) -> Option<&WorthQueryProgramSupportRetirementInventory> {
        match self {
            Self::CurrentBranches(inventory)
            | Self::RetainedInterpretation(inventory)
            | Self::MandatoryCustody(inventory) => Some(inventory),
            Self::UnrosteredProgram { .. }
            | Self::AlreadyRetired { .. }
            | Self::RetirementInProgress { .. }
            | Self::InventoryUnavailable(_) => None,
        }
    }

    pub const fn partial_inventory(
        &self,
    ) -> Option<&WorthQueryProgramSupportPartialRetirementInventory> {
        match self {
            Self::InventoryUnavailable(inventory) => Some(inventory),
            _ => None,
        }
    }
}
