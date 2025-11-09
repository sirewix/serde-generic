// copy-pasted from serde_derive with minor additions
// TODO: figure out copyright notices

#![allow(dead_code)]

#[rustfmt::skip]
pub(crate) mod attr;
#[rustfmt::skip]
pub(crate) mod name;
#[rustfmt::skip]
pub(crate) mod symbol;
#[rustfmt::skip]
pub(crate) mod ctxt;
#[rustfmt::skip]
pub(crate) mod case;

pub use ctxt::Ctxt;
use syn::Type;

pub fn ungroup(mut ty: &Type) -> &Type {
    while let Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}
