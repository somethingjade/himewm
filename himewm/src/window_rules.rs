use crate::init::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
enum MatchType {
    Title,
    Process,
}

#[derive(Deserialize, Serialize)]
enum SetPosition {
    Default,
    Center,
    Position { x: i32, y: i32, w: i32, h: i32 },
}

#[derive(Deserialize, Serialize)]
enum Rule {
    Layout(String),
    Ignore(SetPosition),
}

#[derive(Deserialize, Serialize)]
struct WindowRule {
    match_type: MatchType,
    regex: String,
    rule: Rule,
}

struct UseWindowRule {
    regex: Regex,
    rule: Rule,
}

struct UseWindowRules {
    title_window_rules: Vec<UseWindowRule>,
    process_window_rules: Vec<UseWindowRule>,
}

impl TryFrom<WindowRule> for UseWindowRule {
    type Error = regex::Error;

    fn try_from(value: WindowRule) -> Result<Self, Self::Error> {
        let regex = Regex::new(&value.regex)?;
        return Ok(Self {
            regex,
            rule: value.rule,
        });
    }
}

// fn initialize_window_rules() -> Vec<WindowRule> {
//     let dirs = Directories::new();
// }
