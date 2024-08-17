use std::{
    fmt::Display,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
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
            generate_endpoint(client, item.name).await?;
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

pub async fn generate_endpoint(
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
    let auth_options = docs.auth_options;
    let arguments = docs.arguments;
    let fields = docs.result_fields;

    let consumer = auth_options.consumer;
    let scopes = auth_options.scopes;
    let token = auth_options.token;
    let ssl_required = auth_options.ssl_required;

    println!("name: {name}");
    println!("consumer: {consumer}");
    println!("scopes: {scopes:?}");
    println!("token: {token}");
    println!("ssl_required: {ssl_required}");

    let snake_case_name = name.replace('/', "_");
    println!("snake case name: {snake_case_name}");

    let mut make_uppercase = true;
    let pascal_case_name = name.chars().fold("".to_string(), |mut acc, c| {
        if c == '/' {
            make_uppercase = true;
        } else {
            if make_uppercase {
                acc.push(c.to_ascii_uppercase());
            } else {
                acc.push(c);
            }
            make_uppercase = false;
        }
        acc
    });

    // TODO: if ignored, don't write the argument.
    let consumer_key_type = if consumer == SignatureRequirement::Required {
        "&ConsumerKey"
    } else {
        "Option<&ConsumerKey>"
    };

    let token_type = if token == SignatureRequirement::Required {
        "AccessToken"
    } else {
        "Option<AccessToken>"
    };

    let arguments = generate_arguments(&arguments);
    let output_fields = generate_output_struct_fields(&fields);

    let res = format!(
        "#[derive(Deserialize)]
pub struct {pascal_case_name}Output {{
{output_fields}
}}

/// {name}
///
/// Consumer: {consumer}
///
/// Token: {token}
///
/// Scopes: {scopes:?}
///
/// SSL: {ssl_required}
pub async fn {snake_case_name}(
    consumer_key: {consumer_key_type},
    token: {token_type},
{arguments}
) -> {pascal_case_name}Output {{
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
    let mut res = args.into_iter().fold("".to_string(), |mut acc, arg| {
        // Should we include deprecated arguments?
        let _is_deprecated = arg.is_deprecated;

        let arg_name = &*arg.name;
        let is_required = arg.is_required;

        let returned_type = if is_required {
            "(return type)"
        } else {
            "Option<(return type)>"
        };

        acc.push_str(&format!("\t{arg_name}: {returned_type},\n"));
        acc
    });

    res.pop();
    res
}

fn generate_output_struct_fields(fields: &[Field]) -> String {
    let mut res = fields.into_iter().fold("".to_string(), |mut acc, field| {
        let field_name = &*field.name;
        let returned_type = "(return type)";

        acc.push_str(&format!("\t{field_name}: {returned_type},\n"));
        acc
    });

    res.pop();
    res
}
