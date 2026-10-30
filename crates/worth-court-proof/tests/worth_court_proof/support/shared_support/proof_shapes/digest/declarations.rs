#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProofShapeDigest {
    pub(crate) suite: &'static str,
    pub(crate) entries: Vec<&'static str>,
}

impl ProofShapeDigest {
    pub(crate) fn new(suite: &'static str, entries: Vec<&'static str>) -> Self {
        Self { suite, entries }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn entries(&self) -> &[&'static str] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FailureDigest {
    pub(crate) suite: &'static str,
    pub(crate) entries: Vec<String>,
}

impl FailureDigest {
    pub(crate) fn new<I, Entry>(suite: &'static str, entries: I) -> Self
    where
        I: IntoIterator<Item = Entry>,
        Entry: Into<String>,
    {
        Self {
            suite,
            entries: entries.into_iter().map(Into::into).collect(),
        }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn entries(&self) -> &[String] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransitionDigest {
    pub(crate) suite: &'static str,
    pub(crate) entries: Vec<&'static str>,
}

impl TransitionDigest {
    pub(crate) fn new(suite: &'static str, entries: Vec<&'static str>) -> Self {
        Self { suite, entries }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn entries(&self) -> &[&'static str] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BasisDigest {
    pub(crate) suite: &'static str,
    pub(crate) entries: Vec<&'static str>,
}

impl BasisDigest {
    pub(crate) fn new(suite: &'static str, entries: Vec<&'static str>) -> Self {
        Self { suite, entries }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn entries(&self) -> &[&'static str] {
        &self.entries
    }
}
