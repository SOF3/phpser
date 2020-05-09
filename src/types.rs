use derive_new::new;
use getset::Getters;

/// A serialized PHP value.
#[derive(Debug, Clone)]
pub enum Value<S> {
    /// Corresponds to the `null` type of PHP.
    Null,
    /// Corresponds to the `bool` type of PHP.
    Bool(bool),
    /// Corresponds to the `int` type of PHP.
    Int(i64),
    /// Corresponds to the `float` type of PHP.
    Float(f64),
    /// Corresponds to the `string` type of PHP.
    String(S),
    /// Corresponds to the `array` type of PHP.
    Array(Vec<(ArrayKey<S>, Value<S>)>),
    /// Corresponds to non-`Serializable` objects in PHP.
    Object(Object<S>),
    /// Corresponds to `Serializable` objects in PHP.
    Serializable(Serializable<S>),
    /// Corresponds to an internally-referenced value.
    Reference(Ref),
}

/// The generic array key type
#[derive(Debug, Clone)]
pub enum ArrayKey<S> {
    /// Array key using `int`
    Int(i64),
    /// Array key using `string`
    String(S),
}

/// A non-`Serializable` PHP object.
#[derive(Debug, Clone, Getters, new)]
pub struct Object<S> {
    /// The object class.
    #[getset(get)]
    class: S,
    /// The object properties.
    #[getset(get)]
    properties: Vec<(PropertyName<S>, Value<S>)>,
}

/// The property name of an object.
#[derive(Debug, Clone, Getters, new)]
pub struct PropertyName<S> {
    /// Visibility of the property
    #[getset(get)]
    vis: PropertyVis<S>,
    /// Name of the property
    #[getset(get)]
    name: S,
}

/// The visibility of an object property.
#[derive(Debug, Clone)]
pub enum PropertyVis<S> {
    /// The private visibility.
    ///
    /// The string `S` is the class that declares the property.
    Private(S),
    /// The protected visibility.
    Protected,
    /// The public visibility.
    Public,
}

/// A PHP object that implements `Serializable`.
#[derive(Debug, Clone, Getters, new)]
pub struct Serializable<S> {
    #[getset(get)]
    class: S,
    #[getset(get)]
    data: S,
}

/// A reference to another value in the serialized value tree.
#[derive(Debug, Clone, Copy, new)]
pub struct Ref(usize);
