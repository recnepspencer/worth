impl super::UiOverlayApplicationGeneration {
    pub(crate) fn application(
        &self,
    ) -> Option<
        &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    >{
        match self {
            Self::Prepared(identity) => Some(identity),
            #[cfg(test)]
            Self::Test(_) => None,
        }
    }

    fn semantic_digest(&self) -> u64 {
        match self {
            Self::Prepared(identity) => fold(
                1,
                identity.semantic_package_identity().narrowing_fingerprint(),
            ),
            #[cfg(test)]
            Self::Test(value) => fold(2, *value),
        }
    }
}

impl super::UiBackdropInstanceIdentity {
    pub(crate) fn semantic_digest(self) -> u64 {
        let mut digest = fold(0xcbf2_9ce4_8422_2325_u64, self.declaration().value());
        match self.scope() {
            super::UiOverlayBackdropInstanceScope::SurfaceSingleton => fold(digest, 0),
            super::UiOverlayBackdropInstanceScope::Portal(portal) => {
                digest = fold(digest, 1);
                fold(digest, portal.diagnostic_value())
            }
        }
    }
}

impl super::UiOverlayStackSnapshot {
    pub(crate) fn application(
        &self,
    ) -> Option<
        &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    >{
        self.generation.application()
    }

    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.runtime_surface
    }

    pub(crate) fn contains(
        &self,
        instance: super::UiBackdropInstanceIdentity,
        declaration: worth_ui_dsl::UiBackdropIdentity,
    ) -> bool {
        self.participants.iter().any(|participant| {
            matches!(
                participant,
                super::UiOverlayStackParticipant::Backdrop(row)
                    if row.identity() == instance && row.declaration() == declaration
            )
        })
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let mut digest = self.generation.semantic_digest();
        digest = fold(digest, self.declaration_surface.value());
        digest = fold(digest, self.runtime_surface.diagnostic_value());
        digest = fold(digest, self.presentation.diagnostic_value());
        digest = fold(digest, self.portal_revision);
        digest = fold(digest, self.backdrop_declaration_revision);
        digest = fold(digest, self.extent_revision);
        digest = fold(digest, self.motion_revision.unwrap_or(0));
        digest = fold(digest, self.participants.len() as u64);
        for participant in &self.participants {
            match participant {
                super::UiOverlayStackParticipant::Portal(row) => {
                    digest = fold(digest, 1);
                    digest = fold(digest, row.portal().diagnostic_value());
                    digest = fold(digest, row.declaration().value());
                    digest = fold(digest, row.ordinal().value());
                    digest = fold(digest, row.lifecycle() as u64 + 1);
                }
                super::UiOverlayStackParticipant::Backdrop(row) => {
                    digest = fold(digest, 2);
                    digest = fold(digest, row.identity().semantic_digest());
                    digest = fold(digest, row.declaration().value());
                    digest = fold_text(digest, row.role().as_str());
                    digest = fold(digest, row.role_revision().value());
                }
            }
        }
        digest
    }
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

fn fold_text(mut digest: u64, value: &str) -> u64 {
    digest = fold(digest, value.len() as u64);
    for byte in value.as_bytes() {
        digest = fold(digest, u64::from(*byte));
    }
    digest
}
