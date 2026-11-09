//! One set of aspect markers serves each independently declared class-count fixture.
use worth_query_decl::facade::application_schema::fragment;
pub struct Class1;
impl fragment::Aspect for Class1 {
    const NAME: &'static str = "Class1";
    type Row = ClassValue;
}
pub struct Class2;
impl fragment::Aspect for Class2 {
    const NAME: &'static str = "Class2";
    type Row = ClassValue;
}
pub struct Class3;
impl fragment::Aspect for Class3 {
    const NAME: &'static str = "Class3";
    type Row = ClassValue;
}
pub struct Class4;
impl fragment::Aspect for Class4 {
    const NAME: &'static str = "Class4";
    type Row = ClassValue;
}
pub struct Class5;
impl fragment::Aspect for Class5 {
    const NAME: &'static str = "Class5";
    type Row = ClassValue;
}
pub struct Class6;
impl fragment::Aspect for Class6 {
    const NAME: &'static str = "Class6";
    type Row = ClassValue;
}
pub struct Class7;
impl fragment::Aspect for Class7 {
    const NAME: &'static str = "Class7";
    type Row = ClassValue;
}
pub struct Class8;
impl fragment::Aspect for Class8 {
    const NAME: &'static str = "Class8";
    type Row = ClassValue;
}
pub struct Class9;
impl fragment::Aspect for Class9 {
    const NAME: &'static str = "Class9";
    type Row = ClassValue;
}
pub struct Class10;
impl fragment::Aspect for Class10 {
    const NAME: &'static str = "Class10";
    type Row = ClassValue;
}
pub struct Class11;
impl fragment::Aspect for Class11 {
    const NAME: &'static str = "Class11";
    type Row = ClassValue;
}
pub struct Class12;
impl fragment::Aspect for Class12 {
    const NAME: &'static str = "Class12";
    type Row = ClassValue;
}

pub struct ClassValue(pub u64);
