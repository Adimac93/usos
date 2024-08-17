use std::{
    fmt::Display,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use heck::{ToPascalCase, ToSnakeCase};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tokio::{fs::create_dir_all, io::AsyncWriteExt};

use crate::{
    errors::AppError,
    module_system::{ModuleItem, ModuleItemKind, ModuleItems},
    reference::{Argument, Field, MethodReference, SignatureRequirement},
    UsosUri,
};

const REQUEST_DELAY: Duration = Duration::from_millis(100);
const PLACEHOLDER_TYPE: &str = "(type here)";

struct OutputDirectory;

impl OutputDirectory {
    const BASE: &'static str = "result";

    pub fn base() -> PathBuf {
        PathBuf::new().join(Self::BASE)
    }

    pub fn from_endpoint_path(path: impl Into<String>) -> PathBuf {
        PathBuf::from(format!("{}/{}.rs", Self::BASE, path.into()))
    }
}

pub async fn generate(client: &Client, items: Vec<ModuleItem>) -> Result<(), AppError> {
    for item in items {
        traverse_module_item(client, item).await?;
    }

    Ok(())
}

#[async_recursion::async_recursion]
pub async fn traverse_module_item(client: &Client, item: ModuleItem) -> Result<(), AppError> {
    match item.kind {
        ModuleItemKind::Endpoint => {
            generate_endpoint_file(client, item.name).await?;
            tokio::time::sleep(REQUEST_DELAY).await;
        }
        ModuleItemKind::Module => {
            let nested_items = ModuleItems::get_from_usos(client, item.name).await?;

            for elem in nested_items.into_inner() {
                traverse_module_item(client, elem).await?;
            }
        }
    };

    Ok(())
}

pub async fn generate_endpoint_file(
    client: &Client,
    endpoint_path: impl AsRef<str>,
) -> Result<(), AppError> {
    let endpoint_path = endpoint_path.as_ref();
    let docs = get_usos_endpoint_docs(client, endpoint_path).await?;

    let output_file_path = OutputDirectory::from_endpoint_path(endpoint_path);

    create_dir_all(
        output_file_path
            .parent()
            .context("Output directory path is empty")?,
    )
    .await
    .context("Failed to create directory")?;

    let mut file = tokio::fs::File::create(output_file_path)
        .await
        .context("Failed to create file")?;

    let output = into_code(docs)?;

    file.write(output.as_bytes())
        .await
        .context("Failed to write to file")?;

    println!("{endpoint_path}: success");

    Ok(())
}

async fn get_usos_endpoint_docs(
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

fn into_code(docs: MethodReference) -> Result<String, AppError> {
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

    let authorize_import_lines = if consumer_req == SignatureRequirement::Ignored {
        ""
    } else {
        "use crate::api::oauth1::authorize;\nuse crate::keys::ConsumerKey;\nuse std::collections::HashMap;\n\n"
    };

    let authorize_lines = if consumer_req == SignatureRequirement::Ignored {
        ""
    } else {
        "\tlet authorization = authorize(
        \"POST\",
        &url,
        consumer_key,
        None,
        Some(HashMap::from([
            (\"oauth_callback\".into(), callback.clone()),
        ])),
    );\n"
    };

    let authorize_form_line = if consumer_req == SignatureRequirement::Ignored {
        ""
    } else {
        "\t\t.form(&authorization)\n"
    };

    let res = format!(
        "{authorize_import_lines}use crate::{{
    client::{{UsosUri, CLIENT}},
    util::ToAppResult,
}};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct {output_struct_name} {{
{output_fields}
}}

/// {name}
///
/// Consumer: {consumer_req}
///
/// Token: {token_req}
///
/// Scopes: {scopes:?}
///
/// SSL: {ssl_required}
pub async fn {snake_case_name}(
{consumer_arg_line}{token_arg_line}{other_arguments}) -> crate::Result<{output_struct_name}> {{
    let callback = callback.unwrap_or(\"oob\".into());
    let url = UsosUri::with_path(\"{name}\");
{authorize_lines}
    let body = CLIENT
        .post(&url)
{authorize_form_line}\t\t.send()
        .await?
        .to_app_result()
        .await?
        .json()
        .await?;

    Ok(body)
}}",
    );

    Ok(res)
}

fn generate_arguments(args: &[Argument]) -> String {
    args.into_iter().fold("".to_string(), |mut acc, arg| {
        acc.push_str(&generate_argument(arg));
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

fn generate_output_struct_fields(fields: &[Field]) -> String {
    let mut res = fields.into_iter().fold("".to_string(), |mut acc, field| {
        let field_name = &*field.name;
        let returned_type = PLACEHOLDER_TYPE;

        acc.push_str(&format!("\t{field_name}: {returned_type},\n"));
        acc
    });

    res.pop();
    res
}

fn option(inner: impl AsRef<str>) -> String {
    format!("Option<{}>", inner.as_ref())
}

struct ArgLine;

impl ArgLine {
    fn optional(arg_name: impl AsRef<str>, arg_type: impl AsRef<str>) -> String {
        format!("\t{}: {},\n", arg_name.as_ref(), option(arg_type.as_ref()))
    }

    fn required(arg_name: impl AsRef<str>, arg_type: impl AsRef<str>) -> String {
        format!("\t{}: {},\n", arg_name.as_ref(), arg_type.as_ref())
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
