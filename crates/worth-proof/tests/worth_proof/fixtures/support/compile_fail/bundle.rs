#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompileFailCase {
    family: &'static str,
    path: &'static str,
}

impl CompileFailCase {
    pub const fn new(family: &'static str, path: &'static str) -> Self {
        Self { family, path }
    }

    pub fn family(&self) -> &'static str {
        self.family
    }

    pub fn path(&self) -> &'static str {
        self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileFailBundle {
    suite: &'static str,
    cases: Vec<CompileFailCase>,
}

impl CompileFailBundle {
    pub fn new(suite: &'static str, cases: Vec<CompileFailCase>) -> Self {
        Self { suite, cases }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn cases(&self) -> &[CompileFailCase] {
        &self.cases
    }

    pub fn families(&self) -> Vec<&'static str> {
        let mut families = Vec::new();
        for case in &self.cases {
            if !families.contains(&case.family()) {
                families.push(case.family());
            }
        }
        families
    }

    pub fn contains_family(&self, family: &'static str) -> bool {
        self.cases.iter().any(|case| case.family() == family)
    }

    pub fn cases_for_family(&self, family: &'static str) -> Self {
        let cases = self
            .cases
            .iter()
            .copied()
            .filter(|case| case.family() == family)
            .collect();
        Self::new(self.suite, cases)
    }

    pub fn cases_for_families(&self, families: &[&'static str]) -> Self {
        let cases = self
            .cases
            .iter()
            .copied()
            .filter(|case| families.iter().any(|family| case.family() == *family))
            .collect();
        Self::new(self.suite, cases)
    }
}
