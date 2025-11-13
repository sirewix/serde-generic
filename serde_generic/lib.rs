pub use serde_generic_derive::SerdeGeneric;
pub mod serde;

pub trait SerdeGeneric {
    /// Type list of polymorphic (generic) parameters.
    type Params;
    /// Generic representation of the type.
    type Repr;
    /// Self but with mocked type parameters
    type Mocked: SerdeGeneric;

    fn to_repr(self) -> Self::Repr;
    fn from_repr(_: Self::Repr) -> Self;
    const CONTAINER: serde::Container<Self>;
}

// Product types
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HCons<H, T>(pub H, pub T);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HNil;

// Sum types (coproducts)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HSum<H, T> {
    L(H),
    R(T),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HSumNil {}

// Peano numbers
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Succ<X>(core::marker::PhantomData<X>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zero;

pub trait PeanoNumber {
    const NUMBER: usize;
}

impl PeanoNumber for Zero {
    const NUMBER: usize = 0;
}

impl<P: PeanoNumber> PeanoNumber for Succ<P> {
    const NUMBER: usize = P::NUMBER + 1;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeVar<X>(pub X);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NamedStruct<X>(pub X);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnnamedStruct<X>(pub X);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitStruct;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enum<X>(pub X);

pub trait SerdeFieldAttr<F, I> {
    const FIELD: serde::Field<Self, F>;
}

pub trait SerdeVariantFieldAttr<F, VI, FI> {
    const FIELD: serde::Field<Self, F>;
}

pub trait SerdeVariantAttr<VI> {
    const VARIANT: serde::Variant;
}

pub trait HLen {
    type Len: PeanoNumber;
}
impl<H, T: HLen> HLen for HCons<H, T> {
    type Len = Succ<T::Len>;
}
impl HLen for HNil {
    type Len = Zero;
}

impl<H, T: HLen> HLen for (H, T) {
    type Len = Succ<T::Len>;
}
impl HLen for () {
    type Len = Zero;
}
