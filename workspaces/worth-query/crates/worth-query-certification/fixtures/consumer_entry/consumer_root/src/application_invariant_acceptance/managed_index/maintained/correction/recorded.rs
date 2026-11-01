//! Actual recorded inverse declarations and the fixed Catalog correction dispatcher.
use super::*;
use worth_query_decl::facade::{
    application_aftermath::*, application_query::*, application_schema::*, *,
};
pub struct Note;
pub struct Mover;
pub struct Motion;
const PREIMAGE_BYTES: usize = 4096; // An explicit fixture retention ceiling; oversized values refuse.
fn key(value: &str, identity: &'static str) -> Result<u64, ApplicationValueEncodeDenial> {
    value.parse().map_err(|_| ApplicationValueEncodeDenial::CodecRejected {
        binding_identity: worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(identity),
        reason: "recorded catalog correction keys are unsigned decimal integers",
    })
}
pub(super) fn visit<V: ApplicationCorrectionVisitor<Catalog>>(
    source: std::any::TypeId,
    visitor: V,
) -> Result<V::Output, V> {
    if source == std::any::TypeId::of::<note::Binding>() {
        Ok(dispatch_application_correction::<Catalog, Note, V>(visitor))
    } else if source == std::any::TypeId::of::<mover::Binding>() {
        Ok(dispatch_application_correction::<Catalog, Mover, V>(
            visitor,
        ))
    } else if source == std::any::TypeId::of::<motion::Binding>() {
        Ok(dispatch_application_correction::<Catalog, Motion, V>(
            visitor,
        ))
    } else {
        Err(visitor)
    }
}
impl ApplicationCorrectionBinding<Catalog> for Note {
    const IDENTITY: &'static str = "maintained-catalog.recorded-note.correction.v1";
    type Source = note::Binding;
    type InverseIntent = note::Input;
    type Disclosure = state::Binding;
    fn disclosure_parameters(
        original: &note::Input,
    ) -> Result<ApplicationQueryParameterSet<state::NoteQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(
            <state::Read as ApplicationQueryIntent<Catalog>>::parameters(&state::Read {
                directory: original.directory.clone(),
            }),
        )
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Catalog, note::WriteNote, note::Input, true, false>
    {
        note::WriteNote::reference()
            .definition()
            .no_external_effect()
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Catalog> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(DirectoryNote::reference())],
                PREIMAGE_BYTES,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &note::Input,
        material: ApplicationCorrectionSourceMaterial<'_, Catalog>,
    ) -> Result<note::Input, ApplicationCorrectionDerivationDenial> {
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
        };
        let prior = StringApplicationValueBinding::decode(
            preimage
                .field(&DeclaredPreImageLocus::from_field(
                    DirectoryNote::reference(),
                ))
                .ok_or(ApplicationCorrectionDerivationDenial::MissingPreImage)?,
        )
        .map_err(|_| ApplicationCorrectionDerivationDenial::InvalidPreImage)?;
        Ok(note::Input {
            replacement: prior,
            ..original.clone()
        })
    }
    fn inverse_key(value: &str) -> Result<u64, ApplicationValueEncodeDenial> {
        key(value, Self::IDENTITY)
    }
    fn visit<V: ApplicationCorrectionVisitor<Catalog>>(
        dispatch: ApplicationCorrectionDispatch<Catalog, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
impl ApplicationCorrectionBinding<Catalog> for Mover {
    const IDENTITY: &'static str = "maintained-catalog.recorded-mover.correction.v1";
    type Source = mover::Binding;
    type InverseIntent = mover::Input;
    type Disclosure = state::Binding;
    fn disclosure_parameters(
        original: &mover::Input,
    ) -> Result<ApplicationQueryParameterSet<state::NoteQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(
            <state::Read as ApplicationQueryIntent<Catalog>>::parameters(&state::Read {
                directory: original.directory.clone(),
            }),
        )
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Catalog, mover::MoveName, mover::Input, true, false>
    {
        mover::MoveName::reference()
            .definition()
            .no_external_effect()
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Catalog> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(ItemName::reference())],
                PREIMAGE_BYTES,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &mover::Input,
        material: ApplicationCorrectionSourceMaterial<'_, Catalog>,
    ) -> Result<mover::Input, ApplicationCorrectionDerivationDenial> {
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
        };
        let prior = NameBinding::decode(
            preimage
                .field(&DeclaredPreImageLocus::from_field(ItemName::reference()))
                .ok_or(ApplicationCorrectionDerivationDenial::MissingPreImage)?,
        )
        .map_err(|_| ApplicationCorrectionDerivationDenial::InvalidPreImage)?;
        Ok(mover::Input {
            replacement: prior,
            ..original.clone()
        })
    }
    fn inverse_key(value: &str) -> Result<u64, ApplicationValueEncodeDenial> {
        key(value, Self::IDENTITY)
    }
    fn visit<V: ApplicationCorrectionVisitor<Catalog>>(
        dispatch: ApplicationCorrectionDispatch<Catalog, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}

impl ApplicationCorrectionBinding<Catalog> for Motion {
    const IDENTITY: &'static str = "maintained-catalog.recorded-motion.correction.v1";
    type Source = motion::Binding;
    type InverseIntent = motion::Input;
    type Disclosure = state::Binding;
    fn disclosure_parameters(
        original: &motion::Input,
    ) -> Result<ApplicationQueryParameterSet<state::NoteQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(
            <state::Read as ApplicationQueryIntent<Catalog>>::parameters(&state::Read {
                directory: original.directory.clone(),
            }),
        )
    }
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        Catalog,
        motion::MoveVertex,
        motion::Input,
        true,
        false,
    > {
        motion::MoveVertex::reference()
            .definition()
            .no_external_effect()
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Catalog> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(
                    super::super::geometry::schema::VertexRegion::reference(),
                )],
                PREIMAGE_BYTES,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &motion::Input,
        material: ApplicationCorrectionSourceMaterial<'_, Catalog>,
    ) -> Result<motion::Input, ApplicationCorrectionDerivationDenial> {
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
        };
        let prior = super::super::bounds::CoordinatesBinding::decode(
            preimage
                .field(&DeclaredPreImageLocus::from_field(
                    super::super::geometry::schema::VertexRegion::reference(),
                ))
                .ok_or(ApplicationCorrectionDerivationDenial::MissingPreImage)?,
        )
        .map_err(|_| ApplicationCorrectionDerivationDenial::InvalidPreImage)?;
        Ok(motion::Input {
            replacement: prior,
            ..original.clone()
        })
    }
    fn inverse_key(value: &str) -> Result<u64, ApplicationValueEncodeDenial> {
        key(value, Self::IDENTITY)
    }
    fn visit<V: ApplicationCorrectionVisitor<Catalog>>(
        dispatch: ApplicationCorrectionDispatch<Catalog, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
