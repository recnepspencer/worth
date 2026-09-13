use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryInvariantEnforcement {
    Blocking,
    Advisory,
}

impl WorthQueryInvariantEnforcement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocking => "blocking",
            Self::Advisory => "advisory",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledInvariantExecutionRequirement {
    slot: String,
    family: String,
    version: NonZeroU32,
    enforcement: WorthQueryInvariantEnforcement,
    executor_role: String,
    state_load_families: Vec<String>,
    max_state_facts: usize,
    max_work_units: u64,
    application_invariant:
        Option<crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor>,
}

impl WorthQueryInstalledInvariantExecutionRequirement {
    pub fn new(
        slot: impl Into<String>,
        family: impl Into<String>,
        version: NonZeroU32,
        enforcement: WorthQueryInvariantEnforcement,
        executor_role: impl Into<String>,
        state_load_families: impl IntoIterator<Item = impl Into<String>>,
        max_state_facts: usize,
        max_work_units: u64,
    ) -> Result<Self, &'static str> {
        let slot = canonical(slot.into())?;
        let family = canonical(family.into())?;
        let executor_role = canonical(executor_role.into())?;
        let mut state_load_families = state_load_families
            .into_iter()
            .map(|value| canonical(value.into()))
            .collect::<Result<Vec<_>, _>>()?;
        state_load_families.sort();
        state_load_families.dedup();
        if state_load_families.is_empty() || max_state_facts == 0 || max_work_units == 0 {
            return Err("invalid-invariant-execution-bounds");
        }
        Ok(Self {
            slot,
            family,
            version,
            enforcement,
            executor_role,
            state_load_families,
            max_state_facts,
            max_work_units,
            application_invariant: None,
        })
    }

    pub fn slot(&self) -> &str {
        &self.slot
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn version(&self) -> NonZeroU32 {
        self.version
    }

    pub fn enforcement(&self) -> WorthQueryInvariantEnforcement {
        self.enforcement
    }

    pub fn executor_role(&self) -> &str {
        &self.executor_role
    }

    pub fn state_load_families(&self) -> &[String] {
        &self.state_load_families
    }

    pub fn max_state_facts(&self) -> usize {
        self.max_state_facts
    }

    pub fn max_work_units(&self) -> u64 {
        self.max_work_units
    }

    pub(crate) fn with_application_invariant(
        mut self,
        descriptor: crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor,
    ) -> Self {
        self.application_invariant = Some(descriptor);
        self
    }

    pub fn application_invariant(
        &self,
    ) -> Option<&crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor> {
        self.application_invariant.as_ref()
    }

    pub(crate) fn canonical_parts(&self) -> Vec<String> {
        let mut parts = [
            vec![
                self.slot.clone(),
                self.family.clone(),
                self.version.get().to_string(),
                self.enforcement.as_str().to_owned(),
                self.executor_role.clone(),
                self.max_state_facts.to_string(),
                self.max_work_units.to_string(),
            ],
            self.state_load_families.clone(),
        ]
        .concat();
        if let Some(invariant) = &self.application_invariant {
            parts.push("installed-application-invariant".to_owned());
            parts.extend(invariant.canonical_parts());
        }
        parts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantExecutionContract {
    NotRequired,
    Declared {
        requirements: Vec<WorthQueryInstalledInvariantExecutionRequirement>,
    },
}

impl WorthQueryInvariantExecutionContract {
    pub fn declared(
        requirements: impl IntoIterator<Item = WorthQueryInstalledInvariantExecutionRequirement>,
    ) -> Result<Self, &'static str> {
        let mut requirements = requirements.into_iter().collect::<Vec<_>>();
        requirements.sort_by(|left, right| left.slot().cmp(right.slot()));
        if requirements.is_empty() {
            return Err("empty-invariant-execution-contract");
        }
        if requirements
            .windows(2)
            .any(|pair| pair[0].slot() == pair[1].slot())
        {
            return Err("duplicate-invariant-execution-slot");
        }
        Ok(Self::Declared { requirements })
    }

    pub fn requirements(&self) -> &[WorthQueryInstalledInvariantExecutionRequirement] {
        match self {
            Self::NotRequired => &[],
            Self::Declared { requirements } => requirements,
        }
    }

    pub fn requirement(
        &self,
        slot: &str,
    ) -> Option<&WorthQueryInstalledInvariantExecutionRequirement> {
        self.requirements()
            .iter()
            .find(|requirement| requirement.slot() == slot)
    }
}

fn canonical(value: String) -> Result<String, &'static str> {
    if value.trim().is_empty() || value.trim() != value {
        Err("invalid-invariant-execution-identity")
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_contract_sorts_requirements_and_rejects_duplicate_slots() {
        let first = requirement("a", ["region", "adjacency"]);
        let second = requirement("b", ["region"]);
        let contract =
            WorthQueryInvariantExecutionContract::declared([second.clone(), first.clone()])
                .unwrap();
        assert_eq!(contract.requirements(), &[first.clone(), second.clone()]);
        assert_eq!(
            WorthQueryInvariantExecutionContract::declared([first, second.clone(), second])
                .unwrap_err(),
            "duplicate-invariant-execution-slot"
        );
    }

    #[test]
    fn requirement_canonicalizes_load_families_and_requires_real_bounds() {
        let requirement = requirement("closed-loop", ["region", "adjacency", "region"]);
        assert_eq!(
            requirement.state_load_families(),
            &["adjacency".to_owned(), "region".to_owned()]
        );
        assert_eq!(
            WorthQueryInstalledInvariantExecutionRequirement::new(
                "closed-loop",
                "topology",
                NonZeroU32::new(1).unwrap(),
                WorthQueryInvariantEnforcement::Blocking,
                "graph",
                ["region"],
                0,
                8,
            )
            .unwrap_err(),
            "invalid-invariant-execution-bounds"
        );
    }

    fn requirement(
        slot: &str,
        families: impl IntoIterator<Item = &'static str>,
    ) -> WorthQueryInstalledInvariantExecutionRequirement {
        WorthQueryInstalledInvariantExecutionRequirement::new(
            slot,
            "topology",
            NonZeroU32::new(1).unwrap(),
            WorthQueryInvariantEnforcement::Blocking,
            "graph",
            families,
            4,
            8,
        )
        .unwrap()
    }
}
