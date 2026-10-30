fn main() {
    let mask = worth_foundational::AspectMask::<worth_foundational::MutationMask>::new([
        worth_foundational::CanonicalFieldPath::single(
            worth_foundational::FieldKey::new("count").unwrap(),
        ),
    ]);

    worth_foundational::aspect_mask_digest_preparation_basis(&mask);
}
