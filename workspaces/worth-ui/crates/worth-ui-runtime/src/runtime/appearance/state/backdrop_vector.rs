#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiBackdropAppearanceStateVector {
    basis: UiBackdropAppearanceBasis,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiBackdropAppearanceBasis {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    turn: crate::runtime::observation::UiObservationTurnIdentity,
    source_basis: u64,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    owner_revisions: [u64; 6],
}

impl UiBackdropAppearanceStateVector {
    pub(crate) fn seal(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Self {
        Self {
            basis: UiBackdropAppearanceBasis {
                session: snapshot.session(),
                generation: snapshot.generation().clone(),
                turn: snapshot.turn(),
                source_basis: snapshot.source_basis(),
                surface,
                owner_revisions: [
                    owner_revision(snapshot, worth_ui_dsl::UiAppearanceStateAxis::Operability),
                    owner_revision(snapshot, worth_ui_dsl::UiAppearanceStateAxis::Focus),
                    owner_revision(snapshot, worth_ui_dsl::UiAppearanceStateAxis::Validation),
                    owner_revision(snapshot, worth_ui_dsl::UiAppearanceStateAxis::Selection),
                    owner_revision(snapshot, worth_ui_dsl::UiAppearanceStateAxis::Hover),
                    owner_revision(snapshot, worth_ui_dsl::UiAppearanceStateAxis::Pressed),
                ],
            },
        }
    }

    pub(crate) const fn basis(&self) -> &UiBackdropAppearanceBasis {
        &self.basis
    }

    pub(crate) const fn session(&self) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.basis.session
    }

    pub(crate) const fn generation(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.basis.generation
    }

    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.basis.surface
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        digest = fold(digest, self.basis.session.as_u64());
        digest = fold(
            digest,
            self.basis
                .generation
                .prepared_generation()
                .semantic_package_identity()
                .narrowing_fingerprint(),
        );
        fold(digest, self.basis.surface.diagnostic_value())
    }

    pub(crate) fn evidence_digest(&self) -> u64 {
        let mut digest = self.semantic_digest();
        digest = fold(digest, self.basis.turn.as_u64());
        digest = fold(digest, self.basis.source_basis);
        self.basis.owner_revisions.into_iter().fold(digest, fold)
    }
}

fn owner_revision(
    snapshot: &super::UiAppearanceOwnerSnapshot,
    axis: worth_ui_dsl::UiAppearanceStateAxis,
) -> u64 {
    match axis {
        worth_ui_dsl::UiAppearanceStateAxis::Operability => snapshot
            .operability()
            .map_or(0, |value| value.owner_revision()),
        worth_ui_dsl::UiAppearanceStateAxis::Focus => {
            snapshot.focus().map_or(0, |value| value.owner_revision())
        }
        worth_ui_dsl::UiAppearanceStateAxis::Validation => snapshot
            .validation()
            .map_or(0, |value| value.owner_revision()),
        worth_ui_dsl::UiAppearanceStateAxis::Selection => snapshot
            .selection()
            .map_or(0, |value| value.owner_revision()),
        worth_ui_dsl::UiAppearanceStateAxis::Hover => snapshot
            .pointer_presence()
            .map_or(0, |value| value.owner_revision()),
        worth_ui_dsl::UiAppearanceStateAxis::Pressed => {
            snapshot.pressed().map_or(0, |value| value.owner_revision())
        }
    }
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}
