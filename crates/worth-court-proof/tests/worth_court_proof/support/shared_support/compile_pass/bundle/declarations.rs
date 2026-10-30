#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CompilePassCase {
    pub(crate) family: &'static str,
    pub(crate) path: &'static str,
}

impl CompilePassCase {
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
pub(crate) struct CompilePassBundle {
    pub(crate) suite: &'static str,
    pub(crate) cases: Vec<CompilePassCase>,
}

impl CompilePassBundle {
    pub(crate) fn new(suite: &'static str, cases: Vec<CompilePassCase>) -> Self {
        Self { suite, cases }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn cases(&self) -> &[CompilePassCase] {
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
}
