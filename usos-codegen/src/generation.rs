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
use tokio::{
    fs::{create_dir, create_dir_all, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    errors::AppError,
    module_system::{ModuleItem, ModuleItemKind, ModuleItems},
    reference::{Argument, Field, MethodReference, SignatureRequirement},
    UsosUri,
};

const REQUEST_DELAY: Duration = Duration::from_millis(100);
const PLACEHOLDER_TYPE: &str = "(type here)";
const OMITTED_ARG_NAMES: [&str; 2] = ["format", "callback"];

struct OutputDirectory;

impl OutputDirectory {
    const BASE: &'static str = "result";

    pub fn base() -> PathBuf {
        PathBuf::new().join(Self::BASE)
    }

    pub fn file_from_endpoint_path(path: impl Into<String>) -> PathBuf {
        PathBuf::from(format!("{}/{}.rs", Self::BASE, path.into()))
    }

    pub fn dir_from_endpoint_path(path: impl Into<String>) -> PathBuf {
        PathBuf::from(format!("{}/{}", Self::BASE, path.into()))
    }
}

pub async fn generate(client: &Client, items: Vec<ModuleItem>) -> Result<(), AppError> {
    for item in items {
        // if suitable directory does not exist
        // append to ./result/services.rs file
        // add pub mod apiref;
        // create directory ./result/services

        // repeat for each path of 2 and more:
        // services/apiref

        let path: &Path = item.name.as_ref();
        let mut ancestors: Vec<&Path> = path.ancestors().collect();
        ancestors.reverse();
        println!("{ancestors:?}");
        for elem in (2..ancestors.len()).map(|i| ancestors[i].to_str().unwrap()) {
            println!("{elem}");
            let (module, written) = elem.rsplit_once('/').unwrap();

            try_write_module_file(module, written).await?;
            create_dir_all(OutputDirectory::dir_from_endpoint_path(module))
                .await
                .context("Failed to create directory")?;
        }

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
            // if starting from services/apiref:
            // append to ./result/services/apiref.rs file
            // for every submodule:
            // if suitable directory does not exist
            // add pub mod {submodule_name};
            // create directory ./services/apiref/{submodule_name}
            // and recurse.

            let nested_items = ModuleItems::get_from_usos(client, &*item.name).await?;

            for elem in nested_items.into_inner() {
                let written_submodule = elem.name.rsplit_once('/').unwrap().1;
                try_write_module_file(&*item.name, written_submodule).await?;

                traverse_module_item(client, elem).await?;
            }
        }
    };

    Ok(())
}

pub async fn try_write_module_file(
    module_path: impl AsRef<str>,
    written: impl AsRef<str>,
) -> Result<(), AppError> {
    let module_path = module_path.as_ref();
    let written = written.as_ref();
    let target_dir = OutputDirectory::file_from_endpoint_path(module_path);

    let mut file = OpenOptions::new()
        .append(true)
        .read(true)
        .create(true)
        .open(target_dir)
        .await
        .context("Failed to create file")?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .await
        .context("Failed to read from file")?;

    let content_to_write = format!("pub mod {written};\n");

    if !buf.contains(content_to_write.as_str()) {
        file.write_all(format!("pub mod {written};\n").as_bytes())
            .await
            .context("Failed to write to file")?;
    }

    Ok(())
}

pub async fn generate_endpoint_file(
    client: &Client,
    endpoint_path: impl AsRef<str>,
) -> Result<(), AppError> {
    let endpoint_path = endpoint_path.as_ref();
    let docs = get_usos_endpoint_docs(client, endpoint_path).await?;

    let output_file_path = OutputDirectory::file_from_endpoint_path(endpoint_path);

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

pub(crate) enum CodegenItem {
    Line(String),
    Indent,
    StopIndent,
}

impl CodegenItem {
    pub(crate) fn line(s: impl Into<String>) -> Self {
        Self::Line(s.into())
    }
}

pub(crate) struct Code {
    items: Vec<CodegenItem>,
}

impl Code {
    pub fn new() -> Self {
        Code { items: Vec::new() }
    }

    pub fn inner_mut(&mut self) -> &mut Vec<CodegenItem> {
        &mut self.items
    }

    pub fn line(mut self, s: impl Into<String>) -> Self {
        self.items.push(CodegenItem::line(s));
        self
    }

    pub fn indent(mut self) -> Self {
        self.items.push(CodegenItem::Indent);
        self
    }

    pub fn stop_indent(mut self) -> Self {
        self.items.push(CodegenItem::StopIndent);
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.extend(other.items);
        self
    }

    /// Panics if indent count goes negative.
    pub fn to_code_string(self) -> String {
        let mut tab_count = 0;
        self.items
            .into_iter()
            .fold(String::new(), |mut acc, item| match item {
                CodegenItem::Indent => {
                    tab_count += 1;
                    acc
                }
                CodegenItem::StopIndent => {
                    if tab_count == 0 {
                        panic!("Indent count is negative");
                    }
                    tab_count -= 1;
                    acc
                }
                CodegenItem::Line(line) => {
                    let tabs = "\t".repeat(tab_count);
                    acc.push_str(&format!("{tabs}{line}\n"));
                    acc
                }
            })
    }
}

impl Extend<CodegenItem> for Code {
    fn extend<T: IntoIterator<Item = CodegenItem>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_struct_to_code_gives_coorect_result() {
        let mut code = Code::new();
        code.extend(vec![
            CodegenItem::line("fn foo() -> i32 {"),
            CodegenItem::Indent,
            CodegenItem::line("5"),
            CodegenItem::StopIndent,
            CodegenItem::line("}"),
        ]);

        assert_eq!(code.to_code_string(), "fn foo() -> i32 {\n\t5\n}\n");
    }

    #[test]
    fn code_struct_high_level_api_gives_correct_result() {
        let code = Code::new()
            .line("fn foo() -> i32 {")
            .indent()
            .line("5")
            .stop_indent()
            .line("}");

        assert_eq!(code.to_code_string(), "fn foo() -> i32 {\n\t5\n}\n");
    }

    #[test]
    fn code_struct_nested_gives_correct_result() {
        let struct_code = Code::new()
            .line("struct Foo {")
            .indent()
            .line("first: String,")
            .line("second: i32,")
            .stop_indent()
            .line("}");

        let code = Code::new()
            .line("fn foo() {")
            .indent()
            .merge(struct_code)
            .stop_indent()
            .line("}");

        assert_eq!(
            code.to_code_string(),
            "fn foo() {\n\tstruct Foo {\n\t\tfirst: String,\n\t\tsecond: i32,\n\t}\n}\n"
        )
    }
}
