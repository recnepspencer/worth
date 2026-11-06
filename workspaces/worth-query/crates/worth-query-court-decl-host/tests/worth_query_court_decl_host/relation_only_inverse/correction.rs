//! The inverse declares only the link's relation pre-image, like a CAD body thread.
use super::*;
pub struct Family;
impl ApplicationAuthoredCorrectionFamily<Schema> for Family {
    const IDENTITY: &'static str = "relation-only-inverse.family.v1";
    type Entity = Node;
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Node> {
        ApplicationAuthoredLogicalKey::from_field(Id::at("Id"))
            .expect("the authored key is declared")
    }
}
pub struct UndoSpawn;
impl ApplicationCorrectionBinding<Schema> for UndoSpawn {
    const IDENTITY: &'static str = "relation-only-inverse.undo-spawn.v1";
    type Source = SpawnBinding;
    type InverseIntent = Despawn;
    type Disclosure = query::Binding;
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, SpawnOp, Spawn, true, false> {
        SpawnOp::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Spawn,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        // As in CAD creation: the spawned anchor and an optional co-born part.
        Some(
            ApplicationAuthoredCorrection::retire_created::<Self, Family, Spawned, Despawned>()
                .co_retire_created::<Self, Family, SpawnedPart, DespawnedPart>(),
        )
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::relations_only(
                [DeclaredPreImageRelationLocus::from_relation(
                    Holds::reference(),
                )],
                4096,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &Spawn,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Despawn, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{
            InvalidPreImage, MissingPreImage, UnsupportedSourceMeaning,
        };
        let ApplicationCorrectionSourceMaterial::RecordedInverse(image) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        if original.inverse_fault == 2 {
            // Unrelated is not declared by this inverse, so nothing is disclosed.
            image
                .relation_count(&DeclaredPreImageRelationLocus::from_relation(
                    Unrelated::reference(),
                ))
                .ok_or(MissingPreImage)?;
        }
        let spawned = image
            .output_entity(&<Spawned as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR)
            .ok_or(MissingPreImage)?;
        let part = image
            .output_entity(&<SpawnedPart as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR);
        let locus = DeclaredPreImageRelationLocus::from_relation(Holds::reference());
        if image.relation_count(&locus).ok_or(MissingPreImage)? != 1 {
            return Err(InvalidPreImage);
        }
        let link = image.relation_at(&locus, 0).ok_or(MissingPreImage)?;
        let (from, to) = link.after().ok_or(InvalidPreImage)?;
        // The link is born with its spawned endpoint: it had no prior endpoints,
        // and it holds the part exactly when the optional part was born.
        if link.before().is_some() || from != spawned || part.is_some_and(|part| to != part) {
            return Err(InvalidPreImage);
        }
        Ok(Despawn {
            id: original.id,
            part: part.map(|_| original.id + 1),
            foreign: (original.inverse_fault == 1).then_some(original.id + 1),
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("despawn:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
