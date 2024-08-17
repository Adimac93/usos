use std::fmt::Display;

use inquire::{MultiSelect, Select};
use reqwest::{Client, Response};

use crate::{
    errors::AppError,
    module_system::{ModuleItem, ModuleItemKind, ModuleItems},
    UsosUri,
};

pub async fn prompt_cli(client: &Client) -> Result<Vec<ModuleItem>, AppError> {
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

        let options = ModuleItems::get_from_usos(client, &curr_module).await?;

        answers = MultiSelect::new(prompt, options.into_inner()).prompt()?;

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

    Ok(answers)
}
