use reqwest::Client;

use crate::{
    errors::AppError,
    reference::{Argument, MethodReference, SignatureRequirement},
    UsosUri,
};

use heck::ToSnakeCase;

use super::code::Code;

const PLACEHOLDER_TYPE: &str = "(type here)";
const OMITTED_ARG_NAMES: [&str; 2] = ["format", "callback"];

const PARAMS_VAR: &str = "params";
const URL_VAR: &str = "url";

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

    let consumer_req = docs.auth_options.consumer;
    let scopes = docs.auth_options.scopes;
    let token_req = docs.auth_options.token;
    let ssl_required = docs.auth_options.ssl_required;

    let consumer_arg_line = ArgLine::with_requirement("consumer_key", "&ConsumerKey", consumer_req);
    let token_arg_line = ArgLine::with_requirement("token", "AccessToken", token_req);

    let args = &docs.arguments;
    let other_arguments = for_each_arg(args, generate_function_argument);

    let mut res = Code::new()
        .merge(generate_imports(consumer_req))
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

    if !consumer_req.is_ignored() {
        res = res.line(consumer_arg_line);
    }

    if !token_req.is_ignored() {
        res = res.line(token_arg_line);
    }

    res = res
        .merge(other_arguments)
        .stop_indent()
        .line(") -> crate::Result<Value> {")
        .indent()
        .line(format!("let {URL_VAR} = UsosUri::with_path(\"{name}\");"))
        .merge(generate_param_handling(args, consumer_req, token_req))
        .merge(generate_result())
        .stop_indent()
        .line("}");

    Ok(res.to_code_string())
}

fn for_each_arg<'a>(
    args: impl IntoIterator<Item = &'a Argument>,
    f: impl Fn(&Argument) -> Code,
) -> Code {
    args.into_iter().fold(Code::new(), |mut acc, arg| {
        if !OMITTED_ARG_NAMES.contains(&&*arg.name) {
            acc = acc.merge(f(arg));
        }
        acc
    })
}

fn generate_imports(consumer_req: SignatureRequirement) -> Code {
    let mut res = Code::new();

    if consumer_req != SignatureRequirement::Ignored {
        res = res.line("use crate::keys::ConsumerKey;")
    }

    res.line("use crate::{")
        .indent()
        .line("api::params::Params,")
        .line("client::{UsosUri, CLIENT},")
        .line("util::Process,")
        .stop_indent()
        .line("};")
        .line("use serde_json::Value;")
        .line("")
}

fn generate_function_argument(arg: &Argument) -> Code {
    // Should we include deprecated arguments?
    let _is_deprecated = arg.is_deprecated;

    let line = if arg.is_required {
        ArgLine::required(&*arg.name, PLACEHOLDER_TYPE)
    } else {
        ArgLine::optional(&*arg.name, PLACEHOLDER_TYPE)
    };

    Code::new().line(line)
}

fn generate_param_handling<'a>(
    args: impl IntoIterator<Item = &'a Argument>,
    consumer: SignatureRequirement,
    token: SignatureRequirement,
) -> Code {
    Code::new()
        .line(format!("let mut {PARAMS_VAR} = Params::new();"))
        .line("")
        .merge(for_each_arg(args, generate_add_to_params_map))
        .merge(generate_params_sign(consumer, token))
}

fn generate_add_to_params_map(arg: &Argument) -> Code {
    let arg_name = &*arg.name;
    let add_param_line = format!("{PARAMS_VAR} = {PARAMS_VAR}.add(\"{arg_name}\", {arg_name});");

    let res = if arg.is_required {
        Code::new().line(add_param_line)
    } else {
        Code::new()
            .line(format!("if let Some({arg_name}) = {arg_name} {{"))
            .indent()
            .line(add_param_line)
            .stop_indent()
            .line("}")
    };

    res.line("")
}

fn generate_params_sign(consumer: SignatureRequirement, token: SignatureRequirement) -> Code {
    if consumer.is_ignored() {
        return Code::new();
    }

    let consumer_key_line = if consumer.is_optional() {
        "consumer_key,"
    } else {
        "Some(consumer_key),"
    };

    let token_key_line = match token {
        SignatureRequirement::Ignored => "None,",
        SignatureRequirement::Optional => "token,",
        SignatureRequirement::Required => "Some(token),",
    };

    Code::new()
        .line(format!("{PARAMS_VAR} = {PARAMS_VAR}.sign("))
        .indent()
        .line("\"POST\",")
        .line(format!("&{URL_VAR},"))
        .line(consumer_key_line)
        .line(token_key_line)
        .stop_indent()
        .line(");")
        .line("")
}

fn generate_result() -> Code {
    Code::new()
        .line("let body = CLIENT")
        .indent()
        .line(format!(".post(&{URL_VAR})"))
        .line(format!(".form(&{PARAMS_VAR})"))
        .line(".process_as_json()")
        .line(".await?;")
        .stop_indent()
        .line("Ok(body)")
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
