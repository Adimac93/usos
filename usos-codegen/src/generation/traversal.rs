use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use reqwest::Client;
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    errors::AppError,
    generation::endpoint::{get_usos_endpoint_docs, into_code},
    module_system::{ModuleItem, ModuleItemKind, ModuleItems},
};

const REQUEST_DELAY: Duration = Duration::from_millis(100);

struct OutputDirectory;

impl OutputDirectory {
    const BASE: &'static str = "result";

    /// Functions as both module and endpoint file
    pub fn module_file(path: impl Into<String>) -> PathBuf {
        PathBuf::from(format!("{}/{}.rs", Self::BASE, path.into()))
    }

    pub fn module_dir(path: impl Into<String>) -> PathBuf {
        PathBuf::from(format!("{}/{}", Self::BASE, path.into()))
    }
}

pub(crate) async fn traverse_above(node: impl AsRef<str>) -> Result<(), AppError> {
    let mut acc = "".to_string();
    for written in node.as_ref().split('/') {
        if !acc.is_empty() {
            try_write_module_file(acc.as_str(), written).await?;
            create_dir_all(OutputDirectory::module_dir(acc.as_str()))
                .await
                .context("Failed to create directory")?;
            acc.push('/');
        }

        acc.push_str(written);
    }

    Ok(())
}

#[async_recursion::async_recursion]
pub(crate) async fn traverse_below(client: &Client, node: ModuleItem) -> Result<(), AppError> {
    match node.kind {
        ModuleItemKind::Endpoint => {
            generate_endpoint_file(client, node.name).await?;
            tokio::time::sleep(REQUEST_DELAY).await;
        }
        ModuleItemKind::Module => {
            let nested_items = ModuleItems::get_from_usos(client, &*node.name).await?;

            for elem in nested_items.into_inner() {
                let written_submodule = elem.name.rsplit_once('/');
                debug_assert!(written_submodule.is_some());
                let (_rest_of_path, written_submodule) = written_submodule.unwrap();

                try_write_module_file(&*node.name, written_submodule).await?;

                traverse_below(client, elem).await?;
            }
        }
    };

    Ok(())
}

pub async fn try_write_module_file(
    module_path: impl AsRef<str>,
    written: impl AsRef<str>,
) -> Result<(), AppError> {
    let (module_path, written) = (module_path.as_ref(), written.as_ref());
    let target_dir = OutputDirectory::module_file(module_path);

    let mut file = open_module_file(target_dir)
        .await
        .context("Failed to create file")?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .await
        .context("Failed to read from file")?;

    let content_to_write = format!("pub mod {written};\n");
    if !buf.contains(content_to_write.as_str()) {
        file.write_all(content_to_write.as_bytes())
            .await
            .context("Failed to write to file")?;
    }

    Ok(())
}

async fn open_module_file(path: impl AsRef<Path>) -> Result<File, tokio::io::Error> {
    Ok(OpenOptions::new()
        .append(true)
        .read(true)
        .create(true)
        .open(path.as_ref())
        .await?)
}

pub async fn generate_endpoint_file(
    client: &Client,
    endpoint_path: impl AsRef<str>,
) -> Result<(), AppError> {
    let endpoint_path = endpoint_path.as_ref();
    let docs = get_usos_endpoint_docs(client, endpoint_path).await?;

    let output_file_path = OutputDirectory::module_file(endpoint_path);

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
