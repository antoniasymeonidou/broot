use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io;

use app::{AppStateCmdResult};
use browser_states::BrowserState;
use conf::Conf;
use external::Launchable;

#[derive(Debug, Clone)]
pub struct Verb {
    pub name: String,
    pub exec_pattern: String,
}

pub struct VerbStore {
    verbs: HashMap<String, Verb>,
}

impl Verb {
    pub fn execute(&self, state: &BrowserState) -> io::Result<AppStateCmdResult> {
        let line = match &state.filtered_tree {
            Some(tree) => tree.selected_line(),
            None => state.tree.selected_line(),
        };
        let path = &line.path;
        Ok(match self.exec_pattern.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" => {
                let path = state.tree.selected_line().path.clone();
                let options = state.options.clone();
                AppStateCmdResult::NewState(Box::new(BrowserState::new(path, options)?))
            }
            ":toggle_hidden" => {
                let mut options = state.options.clone();
                options.show_hidden = !options.show_hidden;
                AppStateCmdResult::NewState(Box::new(BrowserState::new(state.tree.root().clone(), options)?))
            }
            ":open" => AppStateCmdResult::Launch(Launchable::opener(path)?),
            ":parent" => {
                match &state.tree.selected_line().path.parent() {
                    Some(path) => {
                        let path = path.to_path_buf();
                        let options = state.options.clone();
                        AppStateCmdResult::NewState(Box::new(BrowserState::new(path, options)?))
                    }
                    None => AppStateCmdResult::DisplayError("no parent found".to_string()),
                }
            }
            ":quit" => AppStateCmdResult::Quit,
            _ => {
                lazy_static! {
                    static ref regex: Regex = Regex::new(r"\{([\w.]+)\}").unwrap();
                }
                // TODO replace token by token and pass an array of string arguments
                let exec = regex
                    .replace_all(&*self.exec_pattern, |caps: &Captures| {
                        match caps.get(1).unwrap().as_str() {
                            "file" => path.to_string_lossy(),
                            _ => Cow::from("-hu?-"),
                        }
                    }).to_string();
                AppStateCmdResult::Launch(Launchable::from(&exec)?)
            }
        })
    }
}

impl VerbStore {
    pub fn new() -> VerbStore {
        VerbStore {
            verbs: HashMap::new(),
        }
    }
    pub fn fill_from_conf(&mut self, conf: &Conf) {
        for verb_conf in &conf.verbs {
            self.verbs.insert(
                verb_conf.invocation.to_owned(),
                Verb {
                    name: verb_conf.name.to_owned(),
                    exec_pattern: verb_conf.execution.to_owned(),
                },
            );
        }
    }
    pub fn get(&self, verb_key: &str) -> Option<&Verb> {
        self.verbs.get(verb_key)
    }
}
