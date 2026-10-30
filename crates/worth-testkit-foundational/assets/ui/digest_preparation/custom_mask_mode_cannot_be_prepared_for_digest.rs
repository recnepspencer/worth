struct CustomMaskMode;

fn main() {
    let mask = worth_foundational::AspectMask::<CustomMaskMode>::whole_aspect();

    worth_foundational::prepare_aspect_mask_for_digest(
        worth_foundational::AspectKey::new("counter").unwrap(),
        mask,
    );
}
