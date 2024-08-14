use anyhow::Context;
use serde_json::Value;
use tokio::io::AsyncWriteExt;

use crate::errors::AppError;

const GENERATED_CODE_DIR: &str = "result/code.rs";

pub async fn generate_from_json_docs(docs: Value) -> Result<(), AppError> {
    let mut file = tokio::fs::File::create(GENERATED_CODE_DIR).await.unwrap();

    let docs = docs.as_object().unwrap();

    let name = docs.get("name").unwrap().as_str().unwrap();
    let auth_options = docs.get("auth_options").unwrap().as_object().unwrap();
    let arguments = docs.get("arguments").unwrap().as_array().unwrap();
    let fields = docs.get("result_fields").unwrap().as_array().unwrap();

    let consumer = auth_options.get("consumer").unwrap().as_str().unwrap();
    let scopes = auth_options.get("scopes").unwrap().as_array().unwrap();
    let token = auth_options.get("token").unwrap().as_str().unwrap();
    let ssl_required = auth_options.get("ssl_required").unwrap().as_bool().unwrap();

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

    let consumer_key_type = if consumer == "required" {
        "&ConsumerKey"
    } else {
        "Option<&ConsumerKey>"
    };

    let token_type = if token == "required" {
        "AccessToken"
    } else {
        "Option<AccessToken>"
    };

    let arguments = generate_arguments(arguments);
    let output_fields = generate_output_struct_fields(fields);

    file.write(
        format!(
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
        )
        .as_bytes(),
    )
    .await
    .context("Failed to write to file")?;

    Ok(())
}

fn generate_arguments(args: &[Value]) -> String {
    let mut res = args.into_iter().fold("".to_string(), |mut acc, val| {
        let arg_data = val.as_object().unwrap();

        // Should we include deprecated arguments?
        let _is_deprecated = arg_data.get("is_deprecated").unwrap().as_bool().unwrap();

        let arg_name = arg_data.get("name").unwrap().as_str().unwrap();
        let is_required = arg_data.get("is_required").unwrap().as_bool().unwrap();

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

fn generate_output_struct_fields(fields: &[Value]) -> String {
    let mut res = fields.into_iter().fold("".to_string(), |mut acc, val| {
        let field_data = val.as_object().unwrap();

        let field_name = field_data.get("name").unwrap().as_str().unwrap();
        let returned_type = "(return type)";

        acc.push_str(&format!("\t{field_name}: {returned_type},\n"));
        acc
    });

    res.pop();
    res
}
