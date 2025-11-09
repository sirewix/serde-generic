use serde_generic::SerdeGeneric;

#[derive(SerdeGeneric)]
struct Boo<X>(u8, String, X);

#[derive(SerdeGeneric, Debug)]
struct Foo<X, Y> {
    #[serde(skip_serializing_if = "String::is_empty", default = "String::new")]
    foo: String,
    boo: u8,
    xoo: X,
    yoo: Y,
}

#[derive(SerdeGeneric, Debug)]
#[serde(rename_all = "camelCase")]
enum Xoo<X, Y> {
    Lek(X),
    Pek,
    Kek { fes_o: String, les_i: Y },
}

#[derive(SerdeGeneric, Debug)]
struct Yoo;

#[test]
fn oij() {
    //use serde_generic::{SerdeVariantAttr, SerdeVariantFieldAttr, Succ, Zero};
    // panic!("{:?}", <Xoo<(), ()> as SerdeVariantAttr<Succ<Zero>>>::VARIANT);
    // panic!("{:?}", <Xoo<(), ()> as SerdeVariantFieldAttr<(), Succ<Zero>, Succ<Zero>>>::FIELD);
    //panic!("{:?}", <Boo<()> as SerdeFieldAttr<String , (((((),),),),)>>::FIELD);
    //panic!("{:?}", <Foo<(), ()> as SerdeFieldAttr<String , (((((),),),),)>>::FIELD);
}

/*
fn asdf() {
  type FooMockedRepr = <Foo<(), ()> as SerdeGeneric>::MockedReprSelf;
  let x: FooMockedRepr = ("hi".into(), 8, P0, P1);
  let y: (String, u8, P0, P1) = x;
}
*/

/*
impl <X, Y> SerdeGeneric for Foo<X, Y> {
  type Params = (X, (Y, ()));
  type MockedSelf = Foo<P1, P2>;
  type MockedReprSelf = (String, u8, P1, P2); // but hlist

  type Repr = (String, u8, X, Y); // but hlist
}

impl <X, Y> Named<Foo<X, Y>, ((),)> for Foo<X, Y> {
  const NAME: &str = "foo";
}

trait Named<T, I> {
    const NAME: &str;
}
*/
