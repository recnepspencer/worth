use crate::application_query::ApplicationQueryRootPathMeaning;

/// Portable scope description. Fresh installation and operation admission grant reads.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationComponentReadMeaning {
    parts: WorthQueryPortableOperationComponentReadParts,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryPortableOperationComponentReadParts {
    pub operation: String,
    pub set: String,
    pub root: String,
    pub member: String,
    pub paths: Vec<ApplicationQueryRootPathMeaning>,
}

impl ApplicationOperationComponentReadMeaning {
    pub fn from_untrusted_parts(parts: WorthQueryPortableOperationComponentReadParts) -> Self {
        Self { parts }
    }

    pub fn operation(&self) -> &str {
        &self.parts.operation
    }

    pub fn set(&self) -> &str {
        &self.parts.set
    }

    pub fn root(&self) -> &str {
        &self.parts.root
    }

    pub fn member(&self) -> &str {
        &self.parts.member
    }

    pub fn paths(&self) -> &[ApplicationQueryRootPathMeaning] {
        &self.parts.paths
    }

    /// Complete descriptive identity; it never names a Native component.
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        let mut part = |text: &str| {
            write!(&mut result, "{}:{text};", text.len()).expect("String write");
        };
        part("operation-component-read-v1");
        part(self.operation());
        part(self.set());
        part(self.root());
        part(self.member());
        part(&self.paths().len().to_string());
        for path in self.paths() {
            path.visit_canonical_parts(&mut part);
        }
        result
    }
}
