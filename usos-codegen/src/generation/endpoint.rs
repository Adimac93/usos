use reqwest::Client;

use crate::{
    errors::AppError,
    reference::{Argument, Field, MethodReference, SignatureRequirement},
    UsosUri,
};

use heck::{ToPascalCase, ToSnakeCase};

use super::code::Code;

const PLACEHOLDER_TYPE: &str = "(type here)";
const OMITTED_ARG_NAMES: [&str; 2] = ["format", "callback"];

pub(super) async fn get_usos_endpoint_docs(
    client: &Client,
    path: impl AsRef<str>,
) -> Result<MethodReference, AppError> {
    Ok(client
        .get(UsosUri::with_path("services/apiref/method"))
        .query(&[("name", path.as_ref())])
        .send()
        .await?
        .json()
        .await?)
}

pub(super) fn into_code(docs: MethodReference) -> Result<String, AppError> {
    let name = docs.name;
    let snake_case_name = name.to_snake_case();
    let output_struct_name = format!("{}Output", name.to_pascal_case());

    let consumer_req = docs.auth_options.consumer;
    let scopes = docs.auth_options.scopes;
    let token_req = docs.auth_options.token;
    let ssl_required = docs.auth_options.ssl_required;

    let consumer_arg_line = ArgLine::with_requirement("consumer_key", "&ConsumerKey", consumer_req);
    let token_arg_line = ArgLine::with_requirement("token", "AccessToken", token_req);

    let other_arguments = generate_arguments(&docs.arguments);
    let output_fields = generate_output_struct_fields(&docs.result_fields);

    // TODO: handle params
    let mut res = Code::new();

    if consumer_req != SignatureRequirement::Ignored {
        res = res
            .line("use crate::api::oauth1::authorize;")
            .line("use crate::keys::ConsumerKey;")
            .line("use std::collections::HashMap;")
    }

    res = res
        .line("use crate::{")
        .indent()
        .line("client::{UsosUri, CLIENT},")
        .line("util::ToAppResult,")
        .stop_indent()
        .line("};")
        .line("use serde::Deserialize;")
        .line("")
        .line("#[derive(Deserialize)]")
        .line(format!("pub struct {output_struct_name} {{"))
        .indent()
        .merge(output_fields)
        .stop_indent()
        .line("}")
        .line("")
        .line(format!("/// {name}"))
        .line("///")
        .line(format!("/// Consumer: {consumer_req}"))
        .line("///")
        .line(format!("/// Token: {token_req}"))
        .line("///")
        .line(format!("/// Scopes: {scopes:?}"))
        .line("///")
        .line(format!("/// SSL: {ssl_required}"))
        .line(format!("pub async fn {snake_case_name} ("))
        .indent();

    if consumer_req != SignatureRequirement::Ignored {
        res = res.line(consumer_arg_line);
    }

    if token_req != SignatureRequirement::Ignored {
        res = res.line(token_arg_line);
    }

    res = res
        .merge(other_arguments)
        .stop_indent()
        .line(format!(") -> crate::Result<{output_struct_name}> {{"))
        .indent()
        .line(format!("let url = UsosUri::with_path(\"{name}\");"));

    if consumer_req != SignatureRequirement::Ignored {
        let consumer_key_line = if consumer_req == SignatureRequirement::Optional {
            "consumer_key.unwrap(),"
        } else {
            "consumer_key,"
        };

        res = res
            .line("")
            .line("let authorization = authorize(")
            .indent()
            .line("\"POST\",")
            .line("&url,")
            .line(consumer_key_line)
            .line("None,")
            .line("Some(HashMap::from([")
            .indent()
            .line("(\"oauth_callback\".into(), callback.clone()),")
            .stop_indent()
            .line("])),")
            .stop_indent()
            .line(")")
            .line("");
    }

    res = res.line("let body = CLIENT").indent().line(".post(&url)");

    if consumer_req != SignatureRequirement::Ignored {
        res = res.line(".form(&authorization)");
    }

    res = res
        .line(".send()")
        .line(".await?")
        .line(".to_app_result()")
        .line(".await?")
        .line(".json()")
        .line(".await?;")
        .stop_indent()
        .line("Ok(body)")
        .stop_indent()
        .line("}");

    Ok(res.to_code_string())
}

fn generate_arguments(args: &[Argument]) -> Code {
    args.into_iter().fold(Code::new(), |mut acc, arg| {
        if !OMITTED_ARG_NAMES.contains(&&*arg.name) {
            acc = acc.line(&generate_argument(arg));
        }
        acc
    })
}

fn generate_argument(arg: &Argument) -> String {
    // Should we include deprecated arguments?
    let _is_deprecated = arg.is_deprecated;

    if arg.is_required {
        ArgLine::required(&*arg.name, PLACEHOLDER_TYPE)
    } else {
        ArgLine::optional(&*arg.name, PLACEHOLDER_TYPE)
    }
}

fn generate_output_struct_fields(fields: &[Field]) -> Code {
    fields.into_iter().fold(Code::new(), |acc, field| {
        let field_name = &*field.name;
        let returned_type = PLACEHOLDER_TYPE;

        acc.line(&format!("{field_name}: {returned_type},"))
    })
}

fn option(inner: impl AsRef<str>) -> String {
    format!("Option<{}>", inner.as_ref())
}

struct ArgLine;

impl ArgLine {
    fn optional(arg_name: impl AsRef<str>, arg_type: impl AsRef<str>) -> String {
        format!("{}: {},", arg_name.as_ref(), option(arg_type.as_ref()))
    }

    fn required(arg_name: impl AsRef<str>, arg_type: impl AsRef<str>) -> String {
        format!("{}: {},", arg_name.as_ref(), arg_type.as_ref())
    }

    fn with_requirement(
        arg_name: impl AsRef<str>,
        arg_type: impl AsRef<str>,
        requirement: SignatureRequirement,
    ) -> String {
        let (arg_name, arg_type) = (arg_name.as_ref(), arg_type.as_ref());

        match requirement {
            SignatureRequirement::Ignored => "".to_string(),
            SignatureRequirement::Optional => ArgLine::optional(arg_name, arg_type),
            SignatureRequirement::Required => ArgLine::required(arg_name, arg_type),
        }
    }
}
