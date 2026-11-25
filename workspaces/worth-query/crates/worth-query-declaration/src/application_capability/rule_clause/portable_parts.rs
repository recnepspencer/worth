use crate::archive_decode_authority::ArchiveDecodeAuthority;
use worth_foundational::facade::AspectValue;

use super::{
    ApplicationCapabilityAcceptedValues, ApplicationCapabilityGraphClause,
    ApplicationCapabilityGraphRequirement, ApplicationCapabilityGraphRule,
    ApplicationCapabilityScopeGuard,
};
use crate::application_capability::{
    ApplicationCapabilityFieldBinding, ApplicationCapabilityPathContextAnchor,
};
use crate::application_schema::ApplicationAuthorizationPath;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityAcceptedValuesParts {
    pub field: ApplicationCapabilityFieldBinding,
    pub values: Vec<AspectValue>,
}

impl ApplicationCapabilityAcceptedValues {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityAcceptedValuesParts,
    ) -> Self {
        Self {
            field: parts.field,
            values: parts.values,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityAcceptedValuesParts {
        WorthQueryPortableApplicationCapabilityAcceptedValuesParts {
            field: self.field.clone(),
            values: self.values.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityScopeGuardParts {
    pub requirements: Vec<ApplicationCapabilityAcceptedValues>,
}

impl ApplicationCapabilityScopeGuard {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityScopeGuardParts,
    ) -> Self {
        Self {
            requirements: parts.requirements,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityScopeGuardParts {
        WorthQueryPortableApplicationCapabilityScopeGuardParts {
            requirements: self.requirements.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityGraphClauseParts {
    pub path: ApplicationAuthorizationPath,
    pub guard: ApplicationCapabilityScopeGuard,
    pub context_anchors: Vec<ApplicationCapabilityPathContextAnchor>,
}

impl ApplicationCapabilityGraphClause {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityGraphClauseParts,
    ) -> Self {
        Self {
            path: parts.path,
            guard: parts.guard,
            context_anchors: parts.context_anchors,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityGraphClauseParts {
        WorthQueryPortableApplicationCapabilityGraphClauseParts {
            path: self.path.clone(),
            guard: self.guard.clone(),
            context_anchors: self.context_anchors.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityGraphRequirementParts {
    pub clauses: Vec<ApplicationCapabilityGraphClause>,
}

impl ApplicationCapabilityGraphRequirement {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityGraphRequirementParts,
    ) -> Self {
        Self {
            clauses: parts.clauses,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityGraphRequirementParts {
        WorthQueryPortableApplicationCapabilityGraphRequirementParts {
            clauses: self.clauses.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationCapabilityGraphRuleParts {
    pub requirements: Vec<ApplicationCapabilityGraphRequirement>,
}

impl ApplicationCapabilityGraphRule {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationCapabilityGraphRuleParts,
    ) -> Self {
        Self {
            requirements: parts.requirements,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationCapabilityGraphRuleParts {
        WorthQueryPortableApplicationCapabilityGraphRuleParts {
            requirements: self.requirements.clone(),
        }
    }
}
