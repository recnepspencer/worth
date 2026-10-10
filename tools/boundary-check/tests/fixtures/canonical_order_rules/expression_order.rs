fn decide(mut values: Vec<TypeId>, other: TypeId) {
 values.sort_by_key(|value| value.type_id());
 values.sort_unstable_by(|left, right| TypeId::of::<A>().cmp(&TypeId::of::<B>()));
 values.iter().min_by_key(|value| value.type_id());
 std::cmp::max(TypeId::of::<A>(), other);
}
