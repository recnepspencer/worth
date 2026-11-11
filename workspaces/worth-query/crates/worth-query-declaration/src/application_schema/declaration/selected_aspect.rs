//! History consumes the already-lowered entity or relation aspect contract.
use super::*;
use crate::application_schema::{
    fragment::AspectSelector, ApplicationFieldBindingOwnerKind, AspectContractRevision,
    AspectIdentity,
};
impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn selected_aspect_contract<Selected: AspectSelector>(
        &self,
        selected: Selected,
    ) -> Result<(AspectIdentity, AspectContractRevision), ApplicationSchemaDeclarationDenial> {
        let installed = selected.installed(&self.member_provenance.fragments.catalog)?;
        self.members
            .iter()
            .find_map(|member| match member {
                ApplicationSchemaMember::Aspect {
                    entity,
                    aspect,
                    identity,
                    revision,
                    ..
                } if selected.owner_kind() == ApplicationFieldBindingOwnerKind::Entity
                    && entity == selected.owner()
                    && aspect == selected.name() =>
                {
                    Some((*identity, *revision))
                }
                ApplicationSchemaMember::RelationAspect { definition }
                    if selected.owner_kind() == ApplicationFieldBindingOwnerKind::Relation
                        && definition.parts().relation == selected.owner()
                        && definition.parts().aspect == selected.name() =>
                {
                    Some((definition.parts().identity, definition.parts().revision))
                }
                _ => None,
            })
            .ok_or_else(|| ApplicationSchemaDeclarationDenial::InvalidRecordMember {
                path: installed.path().to_owned(),
            })
    }
}
