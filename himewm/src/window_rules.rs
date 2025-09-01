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
enum Rule {
    Layout(String),
    StartFloating(SetPosition),
    FloatingPosition(Position),
}

#[derive(Clone)]
pub enum InternalRule {
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

impl From<&InternalRule> for FilterRule {
    fn from(value: &InternalRule) -> Self {
        match value {
            InternalRule::LayoutIdx(_) => return Self::Layout,
            InternalRule::StartFloating(_) => return Self::StartFloating,
            InternalRule::FloatingPosition(_) => return Self::FloatingPosition,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WindowRule {
    match_type: MatchType,
    regex: String,
    rule: Rule,
}

pub struct InternalWindowRule {
    pub regex: Regex,
    pub rule: InternalRule,
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
            Rule::Layout(layout_name) => match layout_idx_map.get(layout_name) {
                Some(i) => InternalRule::LayoutIdx(*i),
                None => continue,
            },
            Rule::StartFloating(set_position) => {
                InternalRule::StartFloating(set_position.to_owned())
            }
            Rule::FloatingPosition(position) => InternalRule::FloatingPosition(position.to_owned()),
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
