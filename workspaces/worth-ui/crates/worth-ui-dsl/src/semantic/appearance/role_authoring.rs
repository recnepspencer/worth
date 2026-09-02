use super::{
    UiAppearanceAspect, UiAppearanceAspectContract, UiAppearanceAspectContractDenial,
    UiAppearanceDecisionPartition, UiAppearancePartitionAdmissionDenial,
    UiAppearancePartitionAuthoring, UiAppearanceRoleApplicability, UiAppearanceRoleDeclaration,
    UiAppearanceRoleDeclarationDenial, UiAppearanceRoleIdentity, UiAppearanceRoleRevision,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceRoleAuthoring {
    identity: UiAppearanceRoleIdentity,
    revision: UiAppearanceRoleRevision,
    applicability: UiAppearanceRoleApplicability,
    partitions: Vec<(UiAppearanceAspect, UiAppearanceDecisionPartition)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAppearanceRoleAuthoringDenial {
    InvalidRevision,
    Partition(UiAppearancePartitionAdmissionDenial),
    AspectContract(UiAppearanceAspectContractDenial),
    Contract(UiAppearanceRoleDeclarationDenial),
}

impl UiAppearanceRoleAuthoring {
    pub fn new(identity: UiAppearanceRoleIdentity) -> Self {
        Self {
            identity,
            revision: UiAppearanceRoleRevision::new(1).expect("one is a valid revision"),
            applicability: UiAppearanceRoleApplicability::AnyComponent,
            partitions: Vec::new(),
        }
    }

    pub fn revision(mut self, revision: UiAppearanceRoleRevision) -> Self {
        self.revision = revision;
        self
    }

    pub fn applies_to(mut self, applicability: UiAppearanceRoleApplicability) -> Self {
        self.applicability = applicability;
        self
    }

    pub fn applies_to_component(self, component: crate::UiDslComponentReference) -> Self {
        self.applies_to(UiAppearanceRoleApplicability::Component(component))
    }

    pub fn applies_to_backdrop(self) -> Self {
        self.applies_to(UiAppearanceRoleApplicability::Backdrop)
    }

    pub fn cover(
        mut self,
        aspect: UiAppearanceAspect,
        partition: UiAppearancePartitionAuthoring,
    ) -> Result<Self, UiAppearanceRoleAuthoringDenial> {
        let partition = partition.compile(aspect).map_err(|denial| {
            UiAppearanceRoleAuthoringDenial::Partition(UiAppearancePartitionAdmissionDenial::new(
                self.identity.clone(),
                aspect,
                aspect.value_kind(),
                denial,
            ))
        })?;
        self.partitions.push((aspect, partition));
        Ok(self)
    }

    pub fn build(self) -> Result<UiAppearanceRoleDeclaration, UiAppearanceRoleAuthoringDenial> {
        let contract = match self.applicability {
            UiAppearanceRoleApplicability::Backdrop => UiAppearanceAspectContract::backdrop(),
            UiAppearanceRoleApplicability::AnyComponent
            | UiAppearanceRoleApplicability::Component(_) => {
                let aspects = self.partitions.iter().map(|(aspect, _)| *aspect);
                UiAppearanceAspectContract::component(aspects, [])
                    .map_err(UiAppearanceRoleAuthoringDenial::AspectContract)?
            }
        };
        UiAppearanceRoleDeclaration::admit(
            self.identity,
            self.revision,
            self.applicability,
            &contract,
            self.partitions,
        )
        .map_err(UiAppearanceRoleAuthoringDenial::Contract)
    }
}
