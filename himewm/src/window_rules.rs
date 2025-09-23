use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
enum MatchType {
    Title,
    Process,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum SetPosition {
    Default,
    Center,
    Position(Position),
}

#[derive(Deserialize, Serialize)]
enum UserRule {
    Layout(String),
    StartFloating(SetPosition),
    FloatingPosition(Position),
}

#[derive(Clone)]
pub enum Rule {
    LayoutIdx(usize),
    StartFloating(SetPosition),
    FloatingPosition(Position),
}

#[derive(PartialEq, Eq, Hash)]
pub enum FilterRule {
    Layout,
    StartFloating,
    FloatingPosition,
}

impl From<&Rule> for FilterRule {
    fn from(value: &Rule) -> Self {
        match value {
            Rule::LayoutIdx(_) => return Self::Layout,
            Rule::StartFloating(_) => return Self::StartFloating,
            Rule::FloatingPosition(_) => return Self::FloatingPosition,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WindowRule {
    match_type: MatchType,
    regex: String,
    rule: UserRule,
}

pub struct InternalWindowRule {
    pub regex: Regex,
    pub rule: Rule,
}

pub struct InternalWindowRules {
    pub title_window_rules: Vec<InternalWindowRule>,
    pub process_window_rules: Vec<InternalWindowRule>,
}

impl Default for InternalWindowRules {
    fn default() -> Self {
        Self {
            title_window_rules: Vec::new(),
            process_window_rules: Vec::new(),
        }
    }
}

pub fn get_internal_window_rules(
    window_rules: &Vec<WindowRule>,
    layout_idx_map: &std::collections::HashMap<String, usize>,
) -> InternalWindowRules {
    let mut ret = InternalWindowRules::default();
    for window_rule in window_rules {
        let rule = match &window_rule.rule {
            UserRule::Layout(layout_name) => match layout_idx_map.get(layout_name) {
                Some(i) => Rule::LayoutIdx(*i),
                None => continue,
            },
            UserRule::StartFloating(set_position) => Rule::StartFloating(set_position.to_owned()),
            UserRule::FloatingPosition(position) => Rule::FloatingPosition(position.to_owned()),
        };
        let internal_window_rule = InternalWindowRule {
            regex: Regex::new(&window_rule.regex).unwrap(),
            rule,
        };
        match window_rule.match_type {
            MatchType::Title => ret.title_window_rules.push(internal_window_rule),
            MatchType::Process => ret.process_window_rules.push(internal_window_rule),
        }
    }
    return ret;
}
