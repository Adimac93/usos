use std::fmt::Display;

use inquire::{MultiSelect, Select};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;

use crate::{errors::AppError, UsosUri};

#[derive(Debug)]
enum GeneratedItems {
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
        let mut answers: Vec<Answer>;
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

            if answers.len() > 1 {
                break;
            } else if answers.len() == 1 {
                let only_answer = answers.pop().unwrap();
                if matches!(only_answer, Answer::Module(_)) {
                    doing_specific_search = true;
                    curr_module = only_answer.into_inner_string();
                } else {
                    break;
                }
            }

            last_answer_empty = answers.is_empty();
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
            endpoints: answers.into_iter().map(|x| x.into_inner_string()).collect(),
        })
    }
}

#[derive(Deserialize)]
pub struct ModuleItems {
    submodules: Vec<String>,
    methods: Vec<String>,
}

#[derive(Debug)]
pub enum Answer {
    Module(String),
    Endpoint(String),
}

impl Answer {
    fn into_inner_string(self) -> String {
        match self {
            Answer::Module(x) => x,
            Answer::Endpoint(x) => x,
        }
    }
}

impl Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Answer::Module(x) => write!(f, "Module: {x}"),
            Answer::Endpoint(x) => write!(f, "Endpoint: {x}"),
        }
    }
}

impl From<ModuleItems> for Vec<Answer> {
    fn from(val: ModuleItems) -> Self {
        let modules = val.submodules.into_iter().map(|x| Answer::Module(x));
        let endpoints = val.methods.into_iter().map(|x| Answer::Endpoint(x));

        modules.chain(endpoints).collect()
    }
}
