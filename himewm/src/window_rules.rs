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

#[derive(Clone, Deserialize, Serialize)]
pub struct InvisibleBorder {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Default for InvisibleBorder {
    fn default() -> Self {
        Self {
            left: 7,
            top: 0,
            right: 7,
            bottom: 7,
        }
    }
}

#[derive(Deserialize, Serialize)]
enum UserRule {
    Layout(String),
    StartFloating(SetPosition),
    FloatingPosition(Position),
    InvisibleBorder(InvisibleBorder),
}

#[derive(Clone)]
pub enum Rule {
    LayoutIdx(usize),
    StartFloating(SetPosition),
    FloatingPosition(Position),
    InvisibleBorder(InvisibleBorder),
}

#[derive(PartialEq, Eq, Hash)]
pub enum FilterRule {
    Layout,
    StartFloating,
    FloatingPosition,
    InvisibleBorder,
}

impl From<&Rule> for FilterRule {
    fn from(value: &Rule) -> Self {
        match value {
            Rule::LayoutIdx(_) => return Self::Layout,
            Rule::StartFloating(_) => return Self::StartFloating,
            Rule::FloatingPosition(_) => return Self::FloatingPosition,
            Rule::InvisibleBorder(_) => return Self::InvisibleBorder,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserWindowRule {
    match_type: MatchType,
    regex: String,
    rule: UserRule,
}

pub struct WindowRule {
    pub regex: Regex,
    pub rule: Rule,
}

pub struct WindowRules {
    pub title_window_rules: Vec<WindowRule>,
    pub process_window_rules: Vec<WindowRule>,
}

impl Default for WindowRules {
    fn default() -> Self {
        Self {
            title_window_rules: Vec::new(),
            process_window_rules: Vec::new(),
        }
    }
}

pub fn get_window_rules(
    user_window_rules: &Vec<UserWindowRule>,
    layout_idx_map: &std::collections::HashMap<String, usize>,
) -> WindowRules {
    let mut ret = WindowRules::default();
    for user_window_rule in user_window_rules {
        let rule = match &user_window_rule.rule {
            UserRule::Layout(layout_name) => match layout_idx_map.get(layout_name) {
                Some(i) => Rule::LayoutIdx(*i),
                None => continue,
            },
            UserRule::StartFloating(set_position) => Rule::StartFloating(set_position.to_owned()),
            UserRule::FloatingPosition(position) => Rule::FloatingPosition(position.to_owned()),
            UserRule::InvisibleBorder(invisible_border) => {
                Rule::InvisibleBorder(invisible_border.to_owned())
            }
        };
        let window_rule = WindowRule {
            regex: Regex::new(&user_window_rule.regex).unwrap(),
            rule,
        };
        match user_window_rule.match_type {
            MatchType::Title => ret.title_window_rules.push(window_rule),
            MatchType::Process => ret.process_window_rules.push(window_rule),
        }
    }
    return ret;
}
