use std::collections::HashMap;

use crate::errors::AppError;

pub(crate) trait ToAppResult
where
    Self: Sized,
{
    async fn to_app_result(self) -> crate::Result<Self>;
}

impl ToAppResult for reqwest::Response {
    async fn to_app_result(self) -> crate::Result<Self> {
        let status = self.status();

        if status.is_client_error() || status.is_server_error() {
            return Err(AppError::http(status, self.text().await?));
        } else {
            return Ok(self);
        }
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::rstest;

    use super::parse_ampersand_params;

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
}
