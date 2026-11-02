//! A separately versioned domain predicate over one genuine projected Source row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationManagedIndexRefinement {
    identity: &'static str,
    version: u32,
}
impl ApplicationManagedIndexRefinement {
    pub const fn row_local(identity: &'static str, version: u32) -> Self {
        Self { identity, version }
    }
    pub fn meaning(self) -> ApplicationManagedIndexRefinementMeaning {
        ApplicationManagedIndexRefinementMeaning::from_untrusted_parts(
            WorthQueryPortableManagedIndexRefinementParts {
                semantic_version: 1,
                identity: self.identity.to_owned(),
                version: self.version,
            },
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationManagedIndexMatchPosture {
    Candidates,
    Refined,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableManagedIndexRefinementParts {
    pub semantic_version: u16,
    pub identity: String,
    pub version: u32,
}
/// All inputs are the same typed probe and this row's complete Source projection.
/// It grants no roster/other-row lookup, external read, tolerance or completeness.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationManagedIndexRefinementMeaning {
    parts: WorthQueryPortableManagedIndexRefinementParts,
}
impl ApplicationManagedIndexRefinementMeaning {
    pub fn from_untrusted_parts(parts: WorthQueryPortableManagedIndexRefinementParts) -> Self {
        Self { parts }
    }
    pub fn parts(&self) -> &WorthQueryPortableManagedIndexRefinementParts {
        &self.parts
    }
    pub fn validate(&self) -> Result<(), super::ApplicationManagedIndexDefinitionDenial> {
        let p = &self.parts;
        if p.semantic_version != 1
            || p.version == 0
            || p.identity.is_empty()
            || p.identity
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(super::ApplicationManagedIndexDefinitionDenial::UnsupportedRefinement);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application_query::{
        ApplicationQueryManagedIndexMeaning as Selection,
        WorthQueryPortableQueryManagedIndexParts as Parts,
    };
    #[test]
    fn refined_mode_never_defaults_missing_or_invalid_contract_to_candidates() {
        let parts = Parts {
            semantic_version: 2,
            index: "Prefix".into(),
            probe_parameter: "probe".into(),
        };
        let missing = Selection::from_untrusted_parts(parts.clone());
        assert_eq!(
            missing.posture(),
            ApplicationManagedIndexMatchPosture::Refined
        );
        assert!(missing.validate().is_err());
        let valid =
            ApplicationManagedIndexRefinement::row_local("domain.exact-name.v1", 1).meaning();
        assert!(
            Selection::from_refinement_parts(parts.clone(), valid.clone())
                .validate()
                .is_ok()
        );
        for identity in ["", "domain wrong", "domain\nwrong"] {
            assert!(Selection::from_refinement_parts(
                parts.clone(),
                ApplicationManagedIndexRefinement::row_local(identity, 1).meaning()
            )
            .validate()
            .is_err());
        }
        assert!(Selection::from_refinement_parts(
            parts.clone(),
            ApplicationManagedIndexRefinement::row_local("domain.exact-name.v1", 0).meaning()
        )
        .validate()
        .is_err());
        let mut old = parts;
        old.semantic_version = 1;
        assert_eq!(
            Selection::from_untrusted_parts(old.clone()).posture(),
            ApplicationManagedIndexMatchPosture::Candidates
        );
        assert!(Selection::from_untrusted_parts(old.clone())
            .validate()
            .is_ok());
        assert!(Selection::from_refinement_parts(old, valid)
            .validate()
            .is_err());
    }
}
