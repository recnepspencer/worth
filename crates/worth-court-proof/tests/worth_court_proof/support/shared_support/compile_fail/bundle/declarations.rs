#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CompileFailCase {
    pub(crate) family: &'static str,
    pub(crate) path: &'static str,
}

impl CompileFailCase {
    pub(crate) const fn new(family: &'static str, path: &'static str) -> Self {
        Self { family, path }
    }

    pub(crate) fn family(&self) -> &'static str {
        self.family
    }

    pub(crate) fn path(&self) -> &'static str {
        self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompileFailBundle {
    pub(crate) suite: &'static str,
    pub(crate) cases: Vec<CompileFailCase>,
}

impl CompileFailBundle {
    pub(crate) fn new(suite: &'static str, cases: Vec<CompileFailCase>) -> Self {
        Self { suite, cases }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn cases(&self) -> &[CompileFailCase] {
        &self.cases
    }

    pub(crate) fn families(&self) -> Vec<&'static str> {
        let mut families = Vec::new();
        for case in &self.cases {
            if !families.contains(&case.family()) {
                families.push(case.family());
            }
        }
        families
    }

    pub(crate) fn contains_family(&self, family: &'static str) -> bool {
        self.cases.iter().any(|case| case.family() == family)
    }

    pub(crate) fn cases_for_family(&self, family: &'static str) -> Self {
        let cases = self
            .cases
            .iter()
            .copied()
            .filter(|case| case.family() == family)
            .collect();
        Self::new(self.suite, cases)
    }

    pub(crate) fn cases_for_families(&self, families: &[&'static str]) -> Self {
        let cases = self
            .cases
            .iter()
            .copied()
            .filter(|case| families.iter().any(|family| case.family() == *family))
            .collect();
        Self::new(self.suite, cases)
    }
}