use std::fmt::Display;

use inquire::{MultiSelect, Select};
use reqwest::{Client, Response};

use crate::{
    errors::AppError,
    module_system::{ModuleItem, ModuleItemKind, ModuleItems},
    UsosUri,
};

#[derive(Debug)]
pub enum GeneratedItems {
    Structs,
    Functions,
    Both,
}

impl Display for GeneratedItems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GeneratedItems::Structs => "Structs",
                GeneratedItems::Functions => "Functions",
                GeneratedItems::Both => "Both",
            }
        )
    }
}

#[derive(Debug)]
pub struct GenerationOptions {
    pub items: GeneratedItems,
    pub endpoints: Vec<String>,
}

impl GenerationOptions {
    pub async fn prompt_cli(client: Client) -> Result<Self, AppError> {
        let mut curr_module = "services".to_string();
        let mut answers: Vec<ModuleItem>;
        let mut doing_specific_search = false;
        let mut last_answer_empty = false;

        loop {
            let prompt = if last_answer_empty {
                "The answer is empty - try again: "
            } else if doing_specific_search {
                "One module chosen - select individual submodules/endpoints: "
            } else {
                "Select modules/endpoints: "
            };

            let res: Response = client
                .get(UsosUri::with_path("services/apiref/module"))
                .query(&[("name", &curr_module)])
                .send()
                .await?;

            let endpoints = res.json::<ModuleItems>().await?.into();

            answers = MultiSelect::new(prompt, endpoints).prompt()?;

            last_answer_empty = answers.is_empty();
            if answers.len() > 1 {
                break;
            } else if answers.len() == 1 {
                let only_answer = answers.pop().unwrap();
                if only_answer.kind == ModuleItemKind::Module {
                    doing_specific_search = true;
                    curr_module = only_answer.name;
                } else {
                    break;
                }
            }
        }

        let items = Select::new(
            "Items to generate",
            vec![
                GeneratedItems::Structs,
                GeneratedItems::Functions,
                GeneratedItems::Both,
            ],
        )
        .prompt()?;

        Ok(GenerationOptions {
            items,
            endpoints: answers.into_iter().map(|x| x.name).collect(),
        })
    }
}
