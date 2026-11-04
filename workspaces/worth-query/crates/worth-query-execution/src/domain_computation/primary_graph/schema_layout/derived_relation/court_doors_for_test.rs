//! Sealed test doors over this module's private members.
use super::*;

impl DerivedRelationLayout {
    #[doc(hidden)]
    pub fn carriers_for_test(&self) -> &Arc<[(Arc<str>, KindId)]> {
        &self.carriers
    }
}

impl DerivedRelationLayout {
    #[doc(hidden)]
    pub fn from_for_test(&self) -> &ApplicationGraphMemberMeaning {
        &self.from
    }
}

impl DerivedRelationLayout {
    #[doc(hidden)]
    pub fn handle_for_test(&self) -> &DerivedRelationHandle {
        &self.handle
    }
}

impl DerivedRelationLayout {
    #[doc(hidden)]
    pub fn to_for_test(&self) -> &ApplicationGraphMemberMeaning {
        &self.to
    }
}
