#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiAppearanceRoleIdentity(Box<str>);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiAppearanceRoleSchemaVersion(u16);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiAppearanceRoleRevision(u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAppearanceRoleApplicability {
    AnyComponent,
    Component(crate::UiDslComponentReference),
    Backdrop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiThemeSlotUse {
    aspect: super::UiAppearanceAspect,
    slot: super::UiThemeSlotIdentity,
    expected_kind: super::UiThemeValueKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceRoleDeclaration {
    role: UiAppearanceRoleIdentity,
    schema: UiAppearanceRoleSchemaVersion,
    revision: UiAppearanceRoleRevision,
    applicability: UiAppearanceRoleApplicability,
    aspect_contract: super::UiAppearanceAspectContract,
    partitions: Box<
        [(
            super::UiAppearanceAspect,
            super::UiAppearanceDecisionPartition,
        )],
    >,
    slot_uses: Box<[UiThemeSlotUse]>,
}

pub type UiAppearanceRole = UiAppearanceRoleDeclaration;
pub type UiAppearanceRoleId = UiAppearanceRoleIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiThemeSlotUseDenial {
    ValueKindMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceRoleDeclarationDenial {
    Empty,
    DuplicateAspect,
    MissingRequiredAspect,
    UnadmittedAspect,
    ResultValueKindMismatch,
    SlotUseCapacityExceeded,
    ApplicabilityContractMismatch,
}

impl UiAppearanceRoleIdentity {
    pub fn new(value: impl Into<Box<str>>) -> Option<Self> {
        let value = value.into();
        (!value.is_empty() && value.len() <= 128 && value.is_ascii()).then_some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl UiAppearanceRoleSchemaVersion {
    pub const fn current() -> Self {
        Self(1)
    }
    pub const fn revision(self) -> u16 {
        self.0
    }
}

impl UiAppearanceRoleRevision {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl UiThemeSlotUse {
    pub fn new(
        aspect: super::UiAppearanceAspect,
        slot: super::UiThemeSlotIdentity,
        expected_kind: super::UiThemeValueKind,
    ) -> Result<Self, UiThemeSlotUseDenial> {
        if aspect.value_kind() != expected_kind {
            return Err(UiThemeSlotUseDenial::ValueKindMismatch);
        }
        Ok(Self {
            aspect,
            slot,
            expected_kind,
        })
    }

    pub const fn aspect(&self) -> super::UiAppearanceAspect {
        self.aspect
    }
    pub fn slot(&self) -> &super::UiThemeSlotIdentity {
        &self.slot
    }
    pub const fn expected_kind(&self) -> super::UiThemeValueKind {
        self.expected_kind
    }
}

impl UiAppearanceRoleDeclaration {
    pub fn authoring(identity: UiAppearanceRoleIdentity) -> super::UiAppearanceRoleAuthoring {
        super::UiAppearanceRoleAuthoring::new(identity)
    }

    pub fn admit(
        role: UiAppearanceRoleIdentity,
        revision: UiAppearanceRoleRevision,
        applicability: UiAppearanceRoleApplicability,
        contract: &super::UiAppearanceAspectContract,
        partitions: impl IntoIterator<
            Item = (
                super::UiAppearanceAspect,
                super::UiAppearanceDecisionPartition,
            ),
        >,
    ) -> Result<Self, UiAppearanceRoleDeclarationDenial> {
        let applicability_matches = matches!(
            (&applicability, contract.applicability()),
            (
                UiAppearanceRoleApplicability::AnyComponent
                    | UiAppearanceRoleApplicability::Component(_),
                super::UiAppearanceAspectApplicability::Component
            ) | (
                UiAppearanceRoleApplicability::Backdrop,
                super::UiAppearanceAspectApplicability::Backdrop
            )
        );
        if !applicability_matches {
            return Err(UiAppearanceRoleDeclarationDenial::ApplicabilityContractMismatch);
        }
        let mut partitions = partitions.into_iter().collect::<Vec<_>>();
        if partitions.is_empty() {
            return Err(UiAppearanceRoleDeclarationDenial::Empty);
        }
        partitions.sort_by_key(|(aspect, _)| *aspect);
        if partitions.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(UiAppearanceRoleDeclarationDenial::DuplicateAspect);
        }
        if partitions
            .iter()
            .any(|(aspect, _)| !contract.admits(*aspect))
        {
            return Err(UiAppearanceRoleDeclarationDenial::UnadmittedAspect);
        }
        if contract
            .required()
            .iter()
            .any(|aspect| !partitions.iter().any(|(covered, _)| covered == aspect))
        {
            return Err(UiAppearanceRoleDeclarationDenial::MissingRequiredAspect);
        }
        let mut slot_uses = Vec::new();
        for (aspect, partition) in &partitions {
            for cell in partition.cells() {
                if cell.result().value_kind() != aspect.value_kind() {
                    return Err(UiAppearanceRoleDeclarationDenial::ResultValueKindMismatch);
                }
                if let Some(slot) = cell.result().slot() {
                    if !slot_uses.iter().any(|slot_use: &UiThemeSlotUse| {
                        slot_use.aspect == *aspect && slot_use.slot == *slot
                    }) {
                        slot_uses.push(UiThemeSlotUse {
                            aspect: *aspect,
                            slot: slot.clone(),
                            expected_kind: cell.result().value_kind(),
                        });
                        if slot_uses.len() > super::UI_APPEARANCE_SLOT_USES_PER_ROLE_CAPACITY {
                            return Err(UiAppearanceRoleDeclarationDenial::SlotUseCapacityExceeded);
                        }
                    }
                }
            }
        }
        Ok(Self {
            role,
            schema: UiAppearanceRoleSchemaVersion::current(),
            revision,
            applicability,
            aspect_contract: contract.clone(),
            partitions: partitions.into_boxed_slice(),
            slot_uses: slot_uses.into_boxed_slice(),
        })
    }

    pub const fn role(&self) -> &UiAppearanceRoleIdentity {
        &self.role
    }
    pub const fn schema(&self) -> UiAppearanceRoleSchemaVersion {
        self.schema
    }
    pub const fn revision(&self) -> UiAppearanceRoleRevision {
        self.revision
    }
    pub const fn applicability(&self) -> &UiAppearanceRoleApplicability {
        &self.applicability
    }
    pub const fn aspect_contract(&self) -> &super::UiAppearanceAspectContract {
        &self.aspect_contract
    }
    pub fn partitions(
        &self,
    ) -> &[(
        super::UiAppearanceAspect,
        super::UiAppearanceDecisionPartition,
    )] {
        &self.partitions
    }
    pub fn slot_uses(&self) -> &[UiThemeSlotUse] {
        &self.slot_uses
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        super::canonical::role_bytes(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_color_slot_may_serve_distinct_admitted_aspects() {
        let contract = super::super::UiAppearanceAspectContract::component(
            [super::super::UiAppearanceAspect::Background],
            [super::super::UiAppearanceAspect::Foreground],
        )
        .unwrap();
        let slot = super::super::UiThemeSlotIdentity::new("surface.color").unwrap();
        let partition = |slot| {
            super::super::UiAppearanceDecisionPartition::compile(
                [],
                [super::super::UiAppearanceDecisionRule::new(
                    [],
                    super::super::UiAppearanceDecisionResult::theme_slot(
                        slot,
                        super::super::UiThemeValueKind::Color,
                    ),
                )],
            )
            .unwrap()
        };
        let declaration = UiAppearanceRoleDeclaration::admit(
            UiAppearanceRoleIdentity::new("test.role").unwrap(),
            UiAppearanceRoleRevision::new(1).unwrap(),
            UiAppearanceRoleApplicability::Component(
                crate::UiDslComponentReference::new("test.component").unwrap(),
            ),
            &contract,
            [
                (
                    super::super::UiAppearanceAspect::Background,
                    partition(slot.clone()),
                ),
                (
                    super::super::UiAppearanceAspect::Foreground,
                    partition(slot),
                ),
            ],
        )
        .unwrap();
        assert_eq!(declaration.slot_uses().len(), 2);
    }

    #[test]
    fn role_admission_enforces_the_slot_use_capacity() {
        use super::super::{UiAppearanceAxisClass::*, UiAppearanceStateAxis::*};
        let mut rules = Vec::new();
        for operability in [
            OperabilityReady,
            OperabilityPending,
            OperabilityOccupied,
            OperabilityDenied,
            OperabilityUnsupported,
            OperabilityStale,
        ] {
            for focus in [
                FocusUnfocused,
                FocusFocused,
                FocusVisible,
                FocusedWindowInactive,
            ] {
                for pressed in [PressedIdle, PressedArmedInside, PressedCapturedOutside] {
                    let slot = super::super::UiThemeSlotIdentity::new(format!(
                        "slot.{operability:?}.{focus:?}.{pressed:?}"
                    ))
                    .unwrap();
                    rules.push(super::super::UiAppearanceDecisionRule::new(
                        [
                            super::super::UiAppearanceAxisPredicate::exact(operability),
                            super::super::UiAppearanceAxisPredicate::exact(focus),
                            super::super::UiAppearanceAxisPredicate::exact(pressed),
                        ],
                        super::super::UiAppearanceDecisionResult::theme_slot(
                            slot,
                            super::super::UiThemeValueKind::Color,
                        ),
                    ));
                }
            }
        }
        let partition = super::super::UiAppearanceDecisionPartition::compile(
            [
                super::super::UiAppearanceAxisDomain::complete(Operability),
                super::super::UiAppearanceAxisDomain::complete(Focus),
                super::super::UiAppearanceAxisDomain::complete(Pressed),
            ],
            rules,
        )
        .unwrap();
        let contract = super::super::UiAppearanceAspectContract::component(
            [super::super::UiAppearanceAspect::Background],
            [],
        )
        .unwrap();
        assert_eq!(
            UiAppearanceRoleDeclaration::admit(
                UiAppearanceRoleIdentity::new("capacity.role").unwrap(),
                UiAppearanceRoleRevision::new(1).unwrap(),
                UiAppearanceRoleApplicability::Component(
                    crate::UiDslComponentReference::new("test.component").unwrap(),
                ),
                &contract,
                [(super::super::UiAppearanceAspect::Background, partition)],
            ),
            Err(UiAppearanceRoleDeclarationDenial::SlotUseCapacityExceeded)
        );
    }

    #[test]
    fn role_applicability_must_match_the_aspect_contract_family() {
        let contract = super::super::UiAppearanceAspectContract::backdrop();
        assert_eq!(
            UiAppearanceRoleDeclaration::admit(
                UiAppearanceRoleIdentity::new("wrong.target.family").unwrap(),
                UiAppearanceRoleRevision::new(1).unwrap(),
                UiAppearanceRoleApplicability::Component(
                    crate::UiDslComponentReference::new("test.component").unwrap(),
                ),
                &contract,
                [],
            ),
            Err(UiAppearanceRoleDeclarationDenial::ApplicabilityContractMismatch)
        );
    }

    #[test]
    fn unconstrained_component_role_still_requires_a_component_contract() {
        let contract = super::super::UiAppearanceAspectContract::backdrop();
        assert_eq!(
            UiAppearanceRoleDeclaration::admit(
                UiAppearanceRoleIdentity::new("wrong.any-component.family").unwrap(),
                UiAppearanceRoleRevision::new(1).unwrap(),
                UiAppearanceRoleApplicability::AnyComponent,
                &contract,
                [],
            ),
            Err(UiAppearanceRoleDeclarationDenial::ApplicabilityContractMismatch)
        );
    }
}
