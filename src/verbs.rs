use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io;

use app::{AppState, AppStateCmdResult};
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
    pub fn execute(&self, state: &AppState) -> io::Result<AppStateCmdResult> {
        let line = match &state.filtered_tree {
            Some(tree) => tree.selected_line(),
            None => state.tree.selected_line(),
        };
        let path = &line.path;
        Ok(match self.exec_pattern.as_ref() {
            ":back" => AppStateCmdResult::PopState,
            ":focus" => AppStateCmdResult::NewRoot(path.clone()),
            ":toggle_hidden" => {
                let mut options = state.options.clone();
                options.show_hidden = !options.show_hidden;
                AppStateCmdResult::NewOptions(options)
            }
            ":open" => AppStateCmdResult::Launch(Launchable::opener(path)?),
            ":parent" => AppStateCmdResult::NewRoot(path.parent().unwrap().to_path_buf()),
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