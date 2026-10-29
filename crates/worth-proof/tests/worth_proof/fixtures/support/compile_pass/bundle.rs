#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilePassCase {
    family: &'static str,
    path: &'static str,
}

impl CompilePassCase {
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
pub struct CompilePassBundle {
    suite: &'static str,
    cases: Vec<CompilePassCase>,
}

impl CompilePassBundle {
    pub fn new(suite: &'static str, cases: Vec<CompilePassCase>) -> Self {
        Self { suite, cases }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn cases(&self) -> &[CompilePassCase] {
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
}
