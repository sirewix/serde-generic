use serde_generic::SerdeGeneric;

#[derive(SerdeGeneric, Debug, Clone, PartialEq)]
struct Boo<X>(u8, String, X);

#[derive(SerdeGeneric, Debug, Clone, PartialEq)]
struct Foo<X, Y> {
    #[serde(skip_serializing_if = "String::is_empty", default = "String::new")]
    foo: String,
    boo: u8,
    xoo: Boo<X>,
    yoo: Y,
}

#[derive(SerdeGeneric, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
enum Xoo<X, Y> {
    Lek(X),
    Pek,
    Kek { fes_o: String, les_i: Y },
}

#[derive(SerdeGeneric, Debug, Clone, PartialEq)]
struct Yoo;

// helper trait to assert type equality
trait TypeEq<A, B> {
    const YES: bool = true;
}
impl<A> TypeEq<A, A> for () {}

#[test]
fn types_derivations() {
    #![allow(path_statements)]
    use serde_generic::*;

    // Yoo
    <() as TypeEq<<Yoo as SerdeGeneric>::Repr, UnitStruct>>::YES;
    <() as TypeEq<<Yoo as SerdeGeneric>::Mocked, Yoo>>::YES;
    <() as TypeEq<<Yoo as SerdeGeneric>::Params, ()>>::YES;

    // Boo
    <() as TypeEq<
        <Boo<bool> as SerdeGeneric>::Repr,
        UnnamedStruct<HCons<u8, HCons<String, HCons<bool, HNil>>>>,
    >>::YES;
    <() as TypeEq<<Boo<bool> as SerdeGeneric>::Mocked, Boo<TypeVar<Zero>>>>::YES;
    <() as TypeEq<<Boo<bool> as SerdeGeneric>::Params, (bool, ())>>::YES;

    // Foo
    <() as TypeEq<
        <Foo<bool, u32> as SerdeGeneric>::Repr,
        NamedStruct<HCons<String, HCons<u8, HCons<Boo<bool>, HCons<u32, HNil>>>>>,
    >>::YES;
    <() as TypeEq<
        <Foo<bool, u32> as SerdeGeneric>::Mocked,
        Foo<TypeVar<Zero>, TypeVar<Succ<Zero>>>,
    >>::YES;
    <() as TypeEq<<Foo<bool, u32> as SerdeGeneric>::Params, (bool, (u32, ()))>>::YES;

    // Xoo
    <() as TypeEq<
        <Xoo<bool, u32> as SerdeGeneric>::Repr,
        Enum<
            HSum<
                UnnamedStruct<HCons<bool, HNil>>,
                HSum<UnitStruct, HSum<NamedStruct<HCons<String, HCons<u32, HNil>>>, HSumNil>>,
            >,
        >,
    >>::YES;
    <() as TypeEq<
        <Xoo<bool, u32> as SerdeGeneric>::Mocked,
        Xoo<TypeVar<Zero>, TypeVar<Succ<Zero>>>,
    >>::YES;
    <() as TypeEq<<Xoo<bool, u32> as SerdeGeneric>::Params, (bool, (u32, ()))>>::YES;

    // composite
    <() as TypeEq<
        <Boo<Boo<bool>> as SerdeGeneric>::Repr,
        UnnamedStruct<HCons<u8, HCons<String, HCons<Boo<bool>, HNil>>>>,
    >>::YES;
    <() as TypeEq<<Boo<Boo<bool>> as SerdeGeneric>::Mocked, Boo<TypeVar<Zero>>>>::YES;
    <() as TypeEq<<Boo<Boo<bool>> as SerdeGeneric>::Params, (Boo<bool>, ())>>::YES;
}
/*
enum Xoo<X, Y> {
    Lek(X),
    Pek,
    Kek { fes_o: String, les_i: Y },
}
*/

/* TODO: test other stuff
    let boo: Boo<bool> = Boo(1, "hi".into(), true);
    let boo_repr: <Boo<bool> as SerdeGeneric>::Repr =
        UnnamedStruct(HCons(1, HCons("hi".into(), HCons(true, HNil))));

    //let _ <<Boo<bool> as SerdeGeneric>::Mocked as Boo<TypeVar<Zero>>> = todo!();
    assert_eq!(boo.clone().to_repr(), boo_repr);
    assert_eq!(boo, Boo::<_>::from_repr(boo_repr));
    //use serde_generic::{SerdeVariantAttr, SerdeVariantFieldAttr, Succ, Zero};
    // panic!("{:?}", <Xoo<(), ()> as SerdeVariantAttr<Succ<Zero>>>::VARIANT);
    // panic!("{:?}", <Xoo<(), ()> as SerdeVariantFieldAttr<(), Succ<Zero>, Succ<Zero>>>::FIELD);
    //panic!("{:?}", <Boo<()> as SerdeFieldAttr<String , (((((),),),),)>>::FIELD);
    //panic!("{:?}", <Foo<(), ()> as SerdeFieldAttr<String , (((((),),),),)>>::FIELD);

*/
