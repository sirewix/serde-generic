/// Represents `serde` struct or enum attribute information.
pub struct Container<T: ?Sized> {
    pub name: MultiName,
    pub transparent: bool,
    pub deny_unknown_fields: bool,
    pub default: Default<T>,
    // pub rename_all_rules: RenameAllRules,
    // pub rename_all_fields_rules: RenameAllRules,
    pub tag: TagType,
    // pub type_from: Option<syn::Type>,
    // pub type_try_from: Option<syn::Type>,
    // pub type_into: Option<syn::Type>,
    // pub remote: Option<syn::Path>,
    pub identifier: Identifier,
    // pub serde_path: Option<syn::Path>,
    pub is_packed: bool,
    /// Error message generated when type can't be deserialized
    pub expecting: Option<&'static str>,
    pub non_exhaustive: bool,
}

/// Styles of representing an enum.
#[derive(Debug, Clone, PartialEq)]
pub enum TagType {
    /// The default.
    ///
    /// ```json
    /// {"variant1": {"key1": "value1", "key2": "value2"}}
    /// ```
    External,

    /// `#[serde(tag = "type")]`
    ///
    /// ```json
    /// {"type": "variant1", "key1": "value1", "key2": "value2"}
    /// ```
    Internal { tag: &'static str },

    /// `#[serde(tag = "t", content = "c")]`
    ///
    /// ```json
    /// {"t": "variant1", "c": {"key1": "value1", "key2": "value2"}}
    /// ```
    Adjacent {
        tag: &'static str,
        content: &'static str,
    },

    /// `#[serde(untagged)]`
    ///
    /// ```json
    /// {"key1": "value1", "key2": "value2"}
    /// ```
    None,
}

/// Whether this enum represents the fields of a struct or the variants of an
/// enum.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Identifier {
    /// It does not.
    No,

    /// This enum represents the fields of a struct. All of the variants must be
    /// unit variants, except possibly one which is annotated with
    /// `#[serde(other)]` and is a newtype variant.
    Field,

    /// This enum represents the variants of an enum. All of the variants must
    /// be unit variants.
    Variant,
}

/// Represents `serde` field attribute information
#[derive(Debug)]
pub struct Field<S: ?Sized, T: ?Sized> {
    pub name: MultiName,
    pub skip_serializing: bool,
    pub skip_deserializing: bool,
    pub skip_serializing_if: Option<(&'static str, fn(&T) -> bool)>,
    pub default: Default<T>,
    pub serialize_with: Option<&'static str>, // TODO: add fn ptrs?
    pub deserialize_with: Option<&'static str>,
    pub getter: Option<(&'static str, fn(&S) -> &T)>, // TODO: case for fn() -> T
    pub flatten: bool,
    pub transparent: bool,
}

#[derive(Debug)]
pub struct MultiName {
    pub serialize: Name,
    pub serialize_renamed: bool,
    pub deserialize: Name,
    pub deserialize_renamed: bool,
    pub deserialize_aliases: &'static [Name],
}

pub type Name = &'static str;

#[derive(Debug)]
pub enum Default<T: ?Sized> {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// The default is given by this function.
    Path(&'static str, fn() -> T),
}

/// Represents `serde` variant attribute information
#[derive(Debug)]
pub struct Variant {
    pub name: MultiName,
    // pub rename_all_rules: RenameAllRules,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
    pub other: bool,
    pub serialize_with: Option<&'static str>, // TODO: add fn ptrs?
    pub deserialize_with: Option<&'static str>,
    pub untagged: bool,
}
