use std::collections::HashMap;

use reqwest::Response;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    api::{auth::AccessToken, errors::UsosError, oauth1::authorize},
    errors::AppError,
    keys::ConsumerKey,
};

pub(crate) trait Process
where
    Self: Sized,
{
    async fn process(self) -> crate::Result<Response>;
    async fn process_as_json<T: DeserializeOwned>(self) -> crate::Result<T>;
}

impl Process for reqwest::RequestBuilder {
    async fn process(self) -> crate::Result<Response> {
        let response = self.send().await?;

        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let error: UsosError = response.json().await?;

            Err(AppError::http(status, error))
        } else {
            Ok(response)
        }
    }

    async fn process_as_json<T: DeserializeOwned>(self) -> crate::Result<T> {
        let response = self.process().await?;
        Ok(response.json().await?)
    }
}

pub(crate) fn parse_ampersand_params(
    text: impl Into<String>,
) -> Result<HashMap<String, String>, AppError> {
    let text = text.into();
    let mut res = HashMap::new();
    let e = AppError::Parse(format!("Invalid ampersand params syntax: {text}"));

    for keyval in text.split('&') {
        if let Some((key, val)) = keyval.split_once('=') {
            if val.contains('=') || key.is_empty() {
                return Err(e);
            }

            res.insert(key.to_string(), val.to_string());
        } else {
            if !keyval.is_empty() {
                return Err(e);
            }
        }
    }

    Ok(res)
}

pub enum Field<'a> {
    One(&'a str),
    Nested(&'a str, Vec<Field<'a>>),
}

pub type Selector = String;

pub(crate) fn format_selector_fields(fields: Vec<Field>) -> Selector {
    fields
        .into_iter()
        .map(|x| match x {
            Field::One(f) => f.into(),
            Field::Nested(base, fields) => {
                format!("{}[{}]", base, format_selector_fields(fields))
            }
        })
        .collect::<Vec<String>>()
        .join("|")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("", &[])]
    #[case("a=b", &[("a", "b")])]
    #[case("abc123=abc123", &[("abc123", "abc123")])]
    #[case("a=b&c=d", &[("a", "b"), ("c", "d")])]
    #[case("a=b&a=c", &[("a", "c")])]
    #[case("a=b&", &[("a", "b")])]
    #[case("&a=b", &[("a", "b")])]
    #[case("&a=b&c=d&", &[("a", "b"), ("c", "d")])]
    #[case("a=b&&c=d", &[("a", "b"), ("c", "d")])]
    #[case("a=", &[("a", "")])]
    fn parse_ampersand_separated_params_is_successful(
        #[case] text: &str,
        #[case] expected: &[(&str, &str)],
    ) {
        let map = parse_ampersand_params(text).unwrap();
        assert_eq!(
            map,
            HashMap::from_iter(
                expected
                    .into_iter()
                    .map(|&x| (x.0.to_string(), x.1.to_string()))
            )
        )
    }

    #[rstest]
    #[case("a==b")]
    #[case("a=b=c&b=c")]
    #[case("=b")]
    #[case("abc")]
    fn parse_ampersand_separated_params_fails(#[case] text: &str) {
        let res = parse_ampersand_params(text);
        assert!(res.is_err())
    }

    #[rstest]
    #[case(vec![], "")]
    #[case(vec![Field::One("a")], "a")]
    #[case(vec![Field::One("a"), Field::One("b")], "a|b")]
    #[case(vec![Field::Nested("a", vec![Field::One("b"), Field::One("c")])], "a[b|c]")]
    #[case(vec![Field::Nested("a", vec![Field::One("b")]), Field::One("c")], "a[b]|c")]
    #[case(vec![Field::One("a"), Field::Nested("b", vec![Field::One("c"), Field::One("d")]), Field::One("e")], "a|b[c|d]|e")]
    fn format_selectors_is_correct(#[case] input: Vec<Field>, #[case] exp: &str) {
        assert_eq!(format_selector_fields(input), exp.to_string())
    }
}
