#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofShapeDigest {
    suite: &'static str,
    entries: Vec<&'static str>,
}

impl ProofShapeDigest {
    pub fn new(suite: &'static str, entries: Vec<&'static str>) -> Self {
        Self { suite, entries }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn entries(&self) -> &[&'static str] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDigest {
    suite: &'static str,
    entries: Vec<String>,
}

impl FailureDigest {
    pub fn new<I, Entry>(suite: &'static str, entries: I) -> Self
    where
        I: IntoIterator<Item = Entry>,
        Entry: Into<String>,
    {
        Self {
            suite,
            entries: entries.into_iter().map(Into::into).collect(),
        }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionDigest {
    suite: &'static str,
    entries: Vec<&'static str>,
}

impl TransitionDigest {
    pub fn new(suite: &'static str, entries: Vec<&'static str>) -> Self {
        Self { suite, entries }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn entries(&self) -> &[&'static str] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasisDigest {
    suite: &'static str,
    entries: Vec<&'static str>,
}

impl BasisDigest {
    pub fn new(suite: &'static str, entries: Vec<&'static str>) -> Self {
        Self { suite, entries }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn entries(&self) -> &[&'static str] {
        &self.entries
    }
}
