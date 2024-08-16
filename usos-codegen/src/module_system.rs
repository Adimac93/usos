use std::fmt::Display;

use serde::Deserialize;

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
    pub name: String,
}

impl ModuleItem {
    pub fn module(name: impl Into<String>) -> Self {
        Self {
            kind: ModuleItemKind::Module,
            name: name.into(),
        }
    }

    pub fn endpoint(name: impl Into<String>) -> Self {
        Self {
            kind: ModuleItemKind::Endpoint,
            name: name.into(),
        }
    }
}

impl Display for ModuleItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.name)
    }
}

impl From<ModuleItems> for Vec<ModuleItem> {
    fn from(val: ModuleItems) -> Self {
        let modules = val.submodules.into_iter().map(|x| ModuleItem::module(x));
        let endpoints = val.methods.into_iter().map(|x| ModuleItem::endpoint(x));

        modules.chain(endpoints).collect()
    }
}

#[derive(Deserialize)]
pub struct ModuleItems {
    submodules: Vec<String>,
    methods: Vec<String>,
}
