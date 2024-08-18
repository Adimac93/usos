use reqwest::Client;
use traversal::{traverse_above, traverse_below};

use crate::{errors::AppError, module_system::ModuleItem};

pub mod code;
pub mod endpoint;
pub mod traversal;

pub async fn generate(client: &Client, items: Vec<ModuleItem>) -> Result<(), AppError> {
    for item in items {
        traverse_above(&*item.name).await?;
        traverse_below(client, item).await?;
    }

    Ok(())
}
