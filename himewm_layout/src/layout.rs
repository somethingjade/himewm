use crate::{common, variant};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Layout {
    monitor_rect: common::Position,
    variants: Vec<variant::Variant>,
    default_variant_idx: usize,
}

impl Layout {
    pub fn new(w: i32, h: i32) -> Self {
        Layout {
            monitor_rect: common::Position::new(0, 0, w, h),
            variants: vec![variant::Variant::new(w, h)],
            default_variant_idx: 0,
        }
    }

    pub fn get_monitor_rect(&self) -> &common::Position {
        &self.monitor_rect
    }

    pub fn set_monitor_rect(&mut self, position: common::Position) {
        self.monitor_rect = position;
    }

    pub fn get_variants(&self) -> &Vec<variant::Variant> {
        &self.variants
    }

    pub fn get_variants_mut(&mut self) -> &mut Vec<variant::Variant> {
        &mut self.variants
    }

    pub fn variants_len(&self) -> usize {
        self.variants.len()
    }

    pub fn default_variant_idx(&self) -> usize {
        self.default_variant_idx
    }

    pub fn set_default_variant_idx(&mut self, i: usize) {
        self.default_variant_idx = i;
    }

    pub fn update_all(&mut self, window_padding: i32, edge_padding: i32) {
        for variant in self.variants.iter_mut() {
            variant.update(window_padding, edge_padding, &self.monitor_rect);
        }
    }

    pub fn get_internal_positions(
        &mut self,
        variant_idx: usize,
        n: usize,
        window_padding: i32,
        edge_padding: i32,
    ) -> &Vec<common::Position> {
        self.variants[variant_idx].get_internal_positions(
            n,
            window_padding,
            edge_padding,
            &self.monitor_rect,
        )
    }
}
