//! The two corrections per entity: `UndoCreate` reverses a creation
//! (`retire_created`, inverse `despawn`), `UndoRetire` reverses a retirement
//! (`restore_retired` from the recorded pre-image, inverse `restore`, a fresh
//! occurrence per ruling 8a.3).
//!
//! The model's co-retirement of `e2` with `e1` (refinement (p)) needs a
//! cascade role on the source binding today; it cannot span two stages, so
//! S1 leaves it out and S2 settles it.

/// The input recovery every correction shares; the inverse and
/// reapplication keys are `inverse:<key>` and `reapply:<key>` unless
/// `$inverse_key` and `$reapplication_key` replace them.
macro_rules! recovery {
    ($source:ty, $inverse:ty) => {
        recovery!(@ $source, $inverse,
            fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
                Ok(format!("inverse:{key}"))
            }
            fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
                Ok(format!("reapply:{key}"))
            }
        );
    };
    (@ $source:ty, $inverse:ty, $inverse_key:item $reapplication_key:item) => {
        fn recover_source_input(
            bytes: &[u8],
            expected: CanonicalDigestId,
            bounds: ValueDecodeBounds,
        ) -> Result<
            ApplicationEncodedInput<<Self::Source as ApplicationMutationBinding<Schema>>::InputBinding>,
            ApplicationRetainedInputDenial,
        > {
            ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, CanonicalValueSeed::<$source>::new(bounds))
        }
        fn recover_inverse_input(
            bytes: &[u8],
            expected: CanonicalDigestId,
            bounds: ValueDecodeBounds,
        ) -> Result<
            ApplicationEncodedInput<<ApplicationInverseBinding<Schema, Self> as ApplicationMutationBinding<Schema>>::InputBinding>,
            ApplicationRetainedInputDenial,
        > {
            ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, CanonicalValueSeed::<$inverse>::new(bounds))
        }
        $inverse_key
        $reapplication_key
        fn visit<V: ApplicationCorrectionVisitor<Schema>>(
            dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
        ) -> V::Output {
            dispatch.conventional()
        }
        fn visit_reapplication<V: ApplicationReapplicationVisitor<Schema>>(
            dispatch: ApplicationReapplicationDispatch<Schema, Self, V>,
        ) -> V::Output {
            dispatch.conventional()
        }
        fn disclosure_parameters(
            _: &$source,
        ) -> Result<ApplicationQueryParameterSet<query::model::Query>, ApplicationCorrectionDerivationDenial>
        {
            Ok(ApplicationQueryParameterSet::new())
        }
    };
}

/// `$name` reverses `$create`'s creation of one member through `$despawn`.
///
/// With `numeric`, the source and inverse bindings key by `u64` and the
/// caller's key must be an unsigned decimal integer (review r1 blocker 2).
macro_rules! undo_create {
    ($name:ident, $create:ident, $despawn:ident, $identity:literal, numeric) => {
        undo_create!(@ $name, $create, $despawn, $identity, [@ $create::Intent, $despawn::Intent,
            fn inverse_key(key: &str) -> Result<u64, ApplicationValueEncodeDenial> {
                key.parse().map_err(|_| ApplicationValueEncodeDenial::CodecRejected {
                    binding_identity: worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared($identity),
                    reason: "numeric DST correction keys are unsigned decimal integers",
                })
            }
            fn reapplication_key(key: &str) -> Result<u64, ApplicationValueEncodeDenial> {
                Self::inverse_key(key)
            }
        ]);
    };
    ($name:ident, $create:ident, $despawn:ident, $identity:literal) => {
        undo_create!(@ $name, $create, $despawn, $identity, [$create::Intent, $despawn::Intent]);
    };
    (@ $name:ident, $create:ident, $despawn:ident, $identity:literal, [$($recovery:tt)*]) => {
        pub struct $name;
        impl ApplicationCorrectionBinding<Schema> for $name {
            const IDENTITY: &'static str = $identity;
            type Source = $create::Binding;
            type InverseIntent = $despawn::Intent;
            type Disclosure = query::model::Binding;
            recovery!($($recovery)*);
            fn source_definition(
            ) -> ApplicationOperationDefinitionBuilder<Schema, $create::Operation, $create::Intent, true, false>
            {
                $create::Operation::reference().definition().no_external_effect()
            }
            fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
                DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>()
            }
            fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
                Some(ApplicationAuthoredCorrection::retire_created::<
                    Self,
                    authored_member::correction::Family,
                    $create::Born,
                    $despawn::Died,
                >())
            }
            fn derive_correction(
                _: &$create::Intent,
                _: ApplicationCorrectionSourceMaterial<'_, Schema>,
            ) -> Result<$despawn::Intent, ApplicationCorrectionDerivationDenial> {
                Ok($despawn::Intent)
            }
        }
    };
}

/// `$name` reverses `$retire`'s retirement of one member through `$restore`,
/// with the value from the recorded pre-image.
macro_rules! undo_retire {
    ($name:ident, $retire:ident, $restore:ident, $identity:literal) => {
        pub struct $name;
        impl ApplicationCorrectionBinding<Schema> for $name {
            const IDENTITY: &'static str = $identity;
            type Source = $retire::Binding;
            type InverseIntent = $restore::Intent;
            type Disclosure = query::model::Binding;
            recovery!($retire::Intent, $restore::Intent);
            fn source_definition(
            ) -> ApplicationOperationDefinitionBuilder<Schema, $retire::Operation, $retire::Intent, true, false>
            {
                $retire::Operation::reference().definition().no_external_effect()
            }
            fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
                DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
                    DeclaredPreImageDemand::new(
                        vec![
                            DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                            DeclaredPreImageLocus::from_field(MemberValue::at("MemberValue")),
                        ],
                        4096,
                    )
                    .with_relations([DeclaredPreImageRelationLocus::from_relation(
                        ModelMember::reference(),
                    )]),
                )
            }
            fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
                Some(ApplicationAuthoredCorrection::restore_retired::<
                    Self,
                    authored_member::correction::Family,
                    $retire::Died,
                    $restore::Born,
                >())
            }
            fn derive_correction(
                _: &$retire::Intent,
                material: ApplicationCorrectionSourceMaterial<'_, Schema>,
            ) -> Result<$restore::Intent, ApplicationCorrectionDerivationDenial> {
                use ApplicationCorrectionDerivationDenial::{
                    InvalidPreImage, MissingPreImage, UnsupportedSourceMeaning,
                };
                let ApplicationCorrectionSourceMaterial::RecordedInverse(fields) = material else {
                    return Err(UnsupportedSourceMeaning);
                };
                let value = fields
                    .field(&DeclaredPreImageLocus::from_field(MemberValue::at("MemberValue")))
                    .ok_or(MissingPreImage)?;
                let value = U64ApplicationValueBinding::decode(value).map_err(|_| InvalidPreImage)?;
                Ok($restore::Intent { value })
            }
        }
    };
}
