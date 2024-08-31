use std::collections::HashMap;

use serde::Serialize;

use crate::keys::ConsumerKey;

use super::{auth::AccessToken, oauth1::authorize};

pub trait IntoParams {
    fn into_params(self) -> HashMap<String, String>;
}

impl IntoParams for () {
    fn into_params(self) -> HashMap<String, String> {
        HashMap::new()
    }
}

impl<T, U> IntoParams for (T, U)
where
    T: Into<String>,
    U: IntoParamString,
{
    fn into_params(self) -> HashMap<String, String> {
        HashMap::from([(self.0.into(), self.1.into_param_string())])
    }
}

impl<T, U, const N: usize> IntoParams for [(T, U); N]
where
    T: Into<String>,
    U: IntoParamString,
{
    fn into_params(self) -> HashMap<String, String> {
        HashMap::from_iter(
            self.into_iter()
                .map(|pair| (pair.0.into(), pair.1.into_param_string())),
        )
    }
}

impl<T, U> IntoParams for HashMap<T, U>
where
    T: Into<String>,
    U: IntoParamString,
{
    fn into_params(self) -> HashMap<String, String> {
        HashMap::from_iter(
            self.into_iter()
                .map(|pair| (pair.0.into(), pair.1.into_param_string())),
        )
    }
}

impl<T, U> IntoParams for Option<HashMap<T, U>>
where
    HashMap<T, U>: IntoParams,
{
    fn into_params(self) -> HashMap<String, String> {
        self.map_or_else(HashMap::new, |x| x.into_params())
    }
}

pub trait IntoParamString {
    fn into_param_string(self) -> String;
}

macro_rules! impl_into_param_string(
    ($($x:ty),*) => {
        $(
            impl IntoParamString for $x {
                fn into_param_string(self) -> String {
                    self.to_string()
                }
            }
        )*
    }
);

impl_into_param_string!(
    String, &str, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl<T> IntoParamString for Option<T>
where
    T: IntoParamString,
{
    fn into_param_string(self) -> String {
        self.map_or_else(String::new, IntoParamString::into_param_string)
    }
}
