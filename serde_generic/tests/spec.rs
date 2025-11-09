// this module both tests and demonstrates how you can use this crate
// to define a spec generator based on SerdeGeneric trait.

use serde_generic::*;
use std::collections::HashSet;

#[derive(SerdeGeneric, Debug)]
struct Foo<X, Y> {
    #[serde(skip_serializing_if = "String::is_empty", default = "String::new")]
    foo: String,
    boo: u8,
    xoo: X,
    coo: Coo<Y>,
}

#[derive(SerdeGeneric, Debug)]
struct Joo<X> {
    joo: X,
}

#[derive(SerdeGeneric, Debug)]
struct Coo<X> {
    coo: X,
}

trait Traverser<H, I, Q> {
    type Collector;
    fn apply(state: &mut Self::Collector);
}

trait Traverse<TR, C, Idx, Q> {
    fn traverse(state: &mut C);
}

impl<H, T, TR, C, Q, QT, I> Traverse<TR, C, Succ<I>, (Q, QT)> for (H, T)
where
    TR: Traverser<H, Succ<I>, Q, Collector = C>,
    T: Traverse<TR, C, I, QT>,
{
    fn traverse(state: &mut C) {
        TR::apply(state);
        T::traverse(state);
    }
}

impl<H, T, TR, C, Q, QT, I> Traverse<TR, C, Succ<I>, (Q, QT)> for HCons<H, T>
where
    TR: Traverser<H, Succ<I>, Q, Collector = C>,
    T: Traverse<TR, C, I, QT>,
{
    fn traverse(state: &mut C) {
        TR::apply(state);
        T::traverse(state);
    }
}

impl<TR, C> Traverse<TR, C, Zero, ()> for () {
    fn traverse(_state: &mut C) {}
}

impl<TR, C> Traverse<TR, C, Zero, ()> for HNil {
    fn traverse(_state: &mut C) {}
}

trait HasSchema<Type> {
    fn schema() -> String;
    fn defs(_: &mut HashSet<String>) {}
}

impl HasSchema<Manual> for String {
    fn schema() -> String {
        "str".into()
    }
}

impl HasSchema<Manual> for u8 {
    fn schema() -> String {
        "byte".into()
    }
}

struct ViaGeneric<T>(core::marker::PhantomData<T>);
struct Manual;

impl<C: SerdeGeneric, CTP, FS> HasSchema<ViaGeneric<(CTP, FS)>> for C
where
    C::Params: HLen
        + Traverse<CollectTypeParams, Vec<String>, <C::Params as HLen>::Len, CTP>
        + Traverse<CollectTypeParamDefs, HashSet<String>, <C::Params as HLen>::Len, CTP>,
    C::MockedReprSelf: BodyDef<C::MockedSelf, FS>,
{
    fn schema() -> String {
        let mut param_list = Vec::new();
        <C::Params as Traverse<CollectTypeParams, _, _, _>>::traverse(&mut param_list);
        format!(
            "{}<{}>",
            Self::CONTAINER.name.serialize,
            &param_list.join(",")
        )
    }

    fn defs(defs: &mut HashSet<String>) {
        let def = format!(
            "{}<{}>={}",
            Self::CONTAINER.name.serialize,
            (0..<C::Params as HLen>::Len::NUMBER)
                .map(|i| format!("X{i}"))
                .collect::<Vec<_>>()
                .join(","),
            <C::MockedReprSelf as BodyDef<C::MockedSelf, FS>>::def(defs)
        );
        defs.insert(def);
        <C::Params as Traverse<CollectTypeParamDefs, _, _, _>>::traverse(defs);
    }
}

impl HasSchema<Manual> for Zero {
    fn schema() -> String {
        "X0".into()
    }
}
impl<P: PeanoNumber> HasSchema<Manual> for Succ<P> {
    fn schema() -> String {
        format!("X{}", 1 + P::NUMBER)
    }
}

// fn variable<N: PeanoNumber>() -> String { format!("X{}", N::NUMBER) }

trait BodyDef<C, FS> {
    fn def(defs: &mut HashSet<String>) -> String;
}

impl<C, X, FS> BodyDef<C, FS> for NamedStruct<X>
where
    X: for<'s> Traverse<
            NamedFieldsTraverser<'s, C>,
            (Vec<(String, String)>, &'s mut HashSet<String>),
            X::Len,
            FS,
        >,
    X: HLen,
    C: SerdeGeneric,
{
    fn def(defs: &mut HashSet<String>) -> String {
        let mut collector = (Vec::new(), defs);
        <X as Traverse<NamedFieldsTraverser<C>, _, _, _>>::traverse(&mut collector);
        format!(
            "{{{}}}",
            collector
                .0
                .iter()
                .map(|(name, ty)| format!("{name}:{ty}"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

#[allow(dead_code)]
struct NamedFieldsTraverser<'s, C>(&'s core::marker::PhantomData<C>);

impl<'s, H, I, C, M> Traverser<H, I, M> for NamedFieldsTraverser<'s, C>
where
    H: HasSchema<M>,
    C: SerdeFieldAttr<H, I>,
{
    type Collector = (Vec<(String, String)>, &'s mut HashSet<String>);
    fn apply((fields, defs): &mut Self::Collector) {
        fields.push((C::FIELD.name.serialize.to_owned(), H::schema()));
        H::defs(defs);
    }
}

struct CollectTypeParams;
impl<H: HasSchema<M>, M, I> Traverser<H, I, M> for CollectTypeParams {
    type Collector = Vec<String>;
    fn apply(param_list: &mut Self::Collector) {
        param_list.push(H::schema());
    }
}

struct CollectTypeParamDefs;
impl<H: HasSchema<M>, M, I> Traverser<H, I, M> for CollectTypeParamDefs {
    type Collector = HashSet<String>;
    fn apply(defs: &mut Self::Collector) {
        H::defs(defs);
    }
}

#[test]
fn test_schema_gen() {
    assert_eq!(
        <Foo<u8, Joo<String>> as HasSchema<_>>::schema(),
        "Foo<byte,Joo<str>>"
    );
    let mut defs = HashSet::new();
    <Foo<u8, Joo<String>> as HasSchema<_>>::defs(&mut defs);
    assert_eq!(
        defs,
        [
            "Foo<X0,X1>={foo:str,boo:byte,xoo:X0,coo:Coo<X1>}",
            "Joo<X0>={joo:X0}",
            "Coo<X0>={coo:X0}",
        ]
        .into_iter()
        .map(String::from)
        .collect::<HashSet<_>>()
    );
}
