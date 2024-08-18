use std::fmt::Display;

use reqwest::{Client, Response};
use serde::Deserialize;

use crate::{errors::AppError, UsosUri};

#[derive(Debug, PartialEq)]
pub enum ModuleItemKind {
    Module,
    Endpoint,
}

impl Display for ModuleItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            ModuleItemKind::Module => "Module",
            ModuleItemKind::Endpoint => "Endpoint",
        };

        write!(f, "{res}")
    }
}

#[derive(Debug)]
pub struct ModuleItem {
    pub kind: ModuleItemKind,
    pub api_path: String,
}

impl ModuleItem {
    pub fn module(name: impl Into<String>) -> Self {
        Self {
            kind: ModuleItemKind::Module,
            api_path: name.into(),
        }
    }

    pub fn endpoint(name: impl Into<String>) -> Self {
        Self {
            kind: ModuleItemKind::Endpoint,
            api_path: name.into(),
        }
    }
}

impl Display for ModuleItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.api_path)
    }
}

impl From<ModuleItemsRaw> for ModuleItems {
    fn from(val: ModuleItemsRaw) -> Self {
        let modules = val.submodules.into_iter().map(|x| ModuleItem::module(x));
        let endpoints = val.methods.into_iter().map(|x| ModuleItem::endpoint(x));

        ModuleItems(modules.chain(endpoints).collect())
    }
}

#[derive(Deserialize)]
pub struct ModuleItemsRaw {
    submodules: Vec<String>,
    methods: Vec<String>,
}

impl ModuleItems {
    pub async fn get_from_usos(
        client: &Client,
        base_module_name: impl AsRef<str>,
    ) -> Result<Self, AppError> {
        let res: Response = client
            .get(UsosUri::with_path("services/apiref/module"))
            .query(&[("name", base_module_name.as_ref())])
            .send()
            .await?;

        Ok(res.json::<ModuleItemsRaw>().await?.into())
    }
}

pub struct ModuleItems(Vec<ModuleItem>);

impl ModuleItems {
    pub fn new(items: Vec<ModuleItem>) -> Self {
        Self(items)
    }

    pub fn into_inner(self) -> Vec<ModuleItem> {
        self.0
    }
}
