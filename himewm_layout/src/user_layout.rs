use crate::{
    position,
    variant
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Deserialize, Serialize)]
pub struct UserLayout<'a> {
    pub w: i32,
    pub h: i32,
    pub default_variant_idx: Vec<usize>,
    #[serde(borrow)]
    pub variants: &'a RawValue
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserVariant {
    pub positions: Vec<Vec<position::Position>>,
    pub end_behaviour: variant::EndBehaviour,
}
