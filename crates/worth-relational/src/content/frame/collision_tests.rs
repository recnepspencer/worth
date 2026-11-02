use super::*;

#[test]
fn equal_claimed_identity_with_unequal_actual_bytes_is_an_internal_conflict() {
    // Corrupt a sealed admission result's body while preserving its claimed id.
    // The production caller charges each chunk before this independent exact check.
    let make = |byte| {
        RelationalContentFrame(Arc::new(AdmittedFrame {
            record: RelationalContentRecord {
                identity: ContentRefId([9; 32]),
                backing: Arc::new(FrameBacking {
                    bytes: vec![byte],
                    memory: None,
                }),
            },
            format: collision_format(),
            payload_offset: 0,
            _metadata: None,
        }))
    };
    let left = make(1);
    let right = make(2);
    assert_eq!(left.identity(), right.identity());
    assert_eq!(
        certify_equal_chunk(left.bytes(), right.bytes()),
        Err(Denial::ContentIdentityConflict)
    );
    assert_eq!(certify_equal_chunk(left.bytes(), left.bytes()), Ok(()));
}

fn collision_format() -> Format {
    #[derive(Debug)]
    struct Codec(ContentFormat);
    impl crate::content::RelationalContentCodec for Codec {
        fn format(&self) -> &ContentFormat {
            &self.0
        }
        fn payload_length(
            &self,
            _: &dyn Any,
            _: &mut super::super::RelationalContentControl<'_, '_, '_>,
        ) -> Result<usize, Denial> {
            Err(Denial::InputTypeMismatch)
        }
        fn encode(
            &self,
            _: &dyn Any,
            _: &mut Sink<'_>,
            _: &mut super::super::RelationalContentControl<'_, '_, '_>,
        ) -> Result<(), Denial> {
            Err(Denial::InputTypeMismatch)
        }
        fn validate(
            &self,
            _: &[u8],
            _: &mut super::super::RelationalContentControl<'_, '_, '_>,
        ) -> Result<(), Denial> {
            Err(Denial::NonCanonicalContent)
        }
        fn verify_encoding(
            &self,
            _: &dyn Any,
            _: &[u8],
            _: &mut super::super::RelationalContentControl<'_, '_, '_>,
        ) -> Result<(), Denial> {
            Err(Denial::EncodingInconsistency)
        }
    }
    let id = worth_foundational::facade::CanonicalDigestId::new([1; 32]);
    let mut formats = crate::content::RelationalContentFormatRegistry::default();
    formats
        .register(
            id,
            Arc::new(Codec(ContentFormat::new("sample-grid", 1).unwrap())),
        )
        .unwrap();
    formats.get(id).unwrap().clone()
}
