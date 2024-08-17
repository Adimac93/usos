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
    cli::GenerationOptions,
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

pub async fn generate(client: &Client, options: GenerationOptions) -> Result<(), AppError> {
    for item in options.module_tree_items {
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

    let consumer_key_type = with_requirement("&ConsumerKey", consumer_req);
    let token_type = with_requirement("AccessToken", token_req);

    let arguments = generate_arguments(&docs.arguments);
    let output_fields = generate_output_struct_fields(&docs.result_fields);

    let res = format!(
        "#[derive(Deserialize)]
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
    consumer_key: {consumer_key_type},
    token: {token_type},
{arguments}) -> {output_struct_name} {{
    let callback = callback.unwrap_or(\"oob\".into());
    let url = UsosUri::with_path(\"{name}\");
    let authorization = authorize(
        <HTTP method, usually GET>,
        &url,
        consumer_key,
        None,
        Some(HashMap::from([
            (\"oauth_callback\".into(), callback.clone()),
        ])),
    );

    let body = CLIENT
        .post(&url)
        .form(&authorization)
        .send()
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

    let arg_name = &*arg.name;
    let is_required = arg.is_required;

    let arg_type = if is_required {
        PLACEHOLDER_TYPE.into()
    } else {
        option(PLACEHOLDER_TYPE)
    };

    format!("\t{arg_name}: {arg_type},\n")
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

fn with_requirement(type_name: impl AsRef<str>, requirement: SignatureRequirement) -> String {
    let type_name = type_name.as_ref();

    match requirement {
        SignatureRequirement::Ignored => "".to_string(),
        SignatureRequirement::Optional => option(type_name),
        SignatureRequirement::Required => type_name.to_string(),
    }
}
