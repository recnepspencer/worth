//! The inverse uses retained member fields and the original typed local intent.
use super::*;
pub struct UndoDelete;
impl ApplicationCorrectionBinding<Schema> for UndoDelete {
    const IDENTITY: &'static str = "source-subgraph.undo-member-delete.v1";
    type Source = DeleteBinding;
    type InverseIntent = Restore;
    type Disclosure = graph_read::source::Binding;
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, DeleteMember, Delete, true, false> {
        DeleteMember::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Delete,
    ) -> Result<
        ApplicationQueryParameterSet<graph_read::source::Query>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("undo:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(ApplicationAuthoredCorrection::restore_retired::<Self, MemberFamily, Deleted, Restored>()
            .owned_member_of(OwnedBy::reference()))
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [
                    DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                    DeclaredPreImageLocus::from_field(Value::at("MemberValue")),
                ],
                1024,
            )
            .with_relations([
                DeclaredPreImageRelationLocus::from_relation(OwnedBy::reference()),
                DeclaredPreImageRelationLocus::from_relation(Head::reference()),
                DeclaredPreImageRelationLocus::from_relation(Operand::reference()),
            ]),
        )
    }
    fn derive_correction(
        original: &Delete,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Restore, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(prior) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let key = StringApplicationValueBinding::decode(
            prior
                .field(&DeclaredPreImageLocus::from_field(MemberKey::at(
                    "MemberKey",
                )))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        let value = U64ApplicationValueBinding::decode(
            prior
                .field(&DeclaredPreImageLocus::from_field(Value::at("MemberValue")))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        Ok(Restore {
            owner: original.owner.clone(),
            key,
            operand: original.operand.clone(),
            value,
        })
    }
}
