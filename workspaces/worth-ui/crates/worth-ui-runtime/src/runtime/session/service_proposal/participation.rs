#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiServiceFamilyParticipation(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiServiceFamilyParticipationDenial {
    DuplicateFamily,
}

impl UiServiceFamilyParticipation {
    pub(in crate::runtime) const EMPTY: Self = Self(0);

    pub(in crate::runtime) fn from_families(
        families: &[crate::capability::UiRuntimeServiceFamily],
    ) -> Result<Self, UiServiceFamilyParticipationDenial> {
        let mut bits = 0;
        for family in families {
            let bit = family_bit(*family);
            if bits & bit != 0 {
                return Err(UiServiceFamilyParticipationDenial::DuplicateFamily);
            }
            bits |= bit;
        }
        Ok(Self(bits))
    }

    pub(in crate::runtime) fn count(self) -> u8 {
        u8::try_from(self.0.count_ones()).expect("an eight-bit set counts at most eight families")
    }

    pub(in crate::runtime) const fn contains(
        self,
        family: crate::capability::UiRuntimeServiceFamily,
    ) -> bool {
        self.0 & family_bit(family) != 0
    }

    pub(in crate::runtime) fn with_family(
        self,
        family: crate::capability::UiRuntimeServiceFamily,
    ) -> Result<Self, UiServiceFamilyParticipationDenial> {
        let bit = family_bit(family);
        if self.0 & bit != 0 {
            return Err(UiServiceFamilyParticipationDenial::DuplicateFamily);
        }
        Ok(Self(self.0 | bit))
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn without(self, settled: Self) -> Self {
        Self(self.0 & !settled.0)
    }
}

const fn family_bit(family: crate::capability::UiRuntimeServiceFamily) -> u8 {
    match family {
        crate::capability::UiRuntimeServiceFamily::Portal => 1 << 0,
        crate::capability::UiRuntimeServiceFamily::Focus => 1 << 1,
        crate::capability::UiRuntimeServiceFamily::Motion => 1 << 2,
        crate::capability::UiRuntimeServiceFamily::CommandRouting => 1 << 3,
        crate::capability::UiRuntimeServiceFamily::Scroll => 1 << 4,
        crate::capability::UiRuntimeServiceFamily::Selection => 1 << 5,
    }
}

#[cfg(any(test, feature = "certification-support"))]
pub(super) fn fixture_service_family_participation(count: usize) -> UiServiceFamilyParticipation {
    UiServiceFamilyParticipation::from_families(
        &crate::capability::UiRuntimeServiceFamily::ALL[..count],
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn participation_preserves_exact_families_and_rejects_duplicates() {
        let families = UiServiceFamilyParticipation::from_families(&[
            crate::capability::UiRuntimeServiceFamily::Portal,
            crate::capability::UiRuntimeServiceFamily::Selection,
        ])
        .unwrap();
        assert_eq!(families.count(), 2);
        assert!(families.contains(crate::capability::UiRuntimeServiceFamily::Portal));
        assert!(!families.contains(crate::capability::UiRuntimeServiceFamily::Motion));
        assert_eq!(
            UiServiceFamilyParticipation::from_families(&[
                crate::capability::UiRuntimeServiceFamily::Scroll,
                crate::capability::UiRuntimeServiceFamily::Scroll,
            ]),
            Err(UiServiceFamilyParticipationDenial::DuplicateFamily)
        );
    }
}
