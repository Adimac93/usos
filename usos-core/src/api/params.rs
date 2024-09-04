use super::{auth::AccessToken, oauth1::authorize};
use crate::keys::ConsumerKey;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

/// A wrapper that contains request form or query parameters.
///
/// Its main purpose is to serve as an intermediate type that allows for
/// compile-time-checked conversions from various collections using the `From` trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params(pub BTreeMap<String, String>);

impl Deref for Params {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Params {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<()> for Params {
    fn from(_: ()) -> Self {
        Self(BTreeMap::new())
    }
}

impl<T, U> From<(T, U)> for Params
where
    T: Into<String>,
    U: Into<ParamString>,
{
    fn from(value: (T, U)) -> Self {
        Self(BTreeMap::from([(value.0.into(), value.1.into().0)]))
    }
}

impl<T, U, const N: usize> From<[(T, U); N]> for Params
where
    T: Into<String> + Ord,
    U: Into<ParamString>,
{
    fn from(value: [(T, U); N]) -> Self {
        Self(BTreeMap::from_iter(
            value
                .into_iter()
                .map(|pair| (pair.0.into(), pair.1.into().0)),
        ))
    }
}

impl<T, U, const N: usize> From<&[(T, U); N]> for Params
where
    T: Into<String> + Clone,
    U: Into<String> + Clone,
{
    fn from(value: &[(T, U); N]) -> Self {
        Self(BTreeMap::from_iter(
            value
                .into_iter()
                .map(|pair| (pair.0.clone().into(), pair.1.clone().into())),
        ))
    }
}

impl<T, U> From<BTreeMap<T, U>> for Params
where
    T: Into<String>,
    U: Into<ParamString>,
{
    fn from(value: BTreeMap<T, U>) -> Self {
        Self(BTreeMap::from_iter(
            value
                .into_iter()
                .map(|pair| (pair.0.into(), pair.1.into().0)),
        ))
    }
}

impl<T, U> From<Option<BTreeMap<T, U>>> for Params
where
    BTreeMap<T, U>: Into<Params>,
{
    fn from(value: Option<BTreeMap<T, U>>) -> Self {
        value.map_or_else(|| Self(BTreeMap::new()), Into::into)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParamString(String);

impl Deref for ParamString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! impl_into_param_string(
    ($($x:ty),*) => {
        $(
            impl From<$x> for ParamString {
                fn from(value: $x) -> Self {
					Self(value.to_string())
				}
            }
        )*
    }
);

impl_into_param_string!(
    String, &str, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
