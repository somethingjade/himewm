use crate::{position, user_layout, variant, variants_container};

#[derive(Clone, Debug)]
pub struct Layout {
    monitor_rect: position::Position,
    variants: variants_container::VariantsContainer<variant::Variant>,
    default_variant_idx: Vec<usize>,
}

impl Layout {
    pub fn new(w: i32, h: i32) -> Self {
        Self {
            monitor_rect: position::Position::new(0, 0, w, h),
            variants: variants_container::VariantsContainer::Variants(Vec::new()),
            default_variant_idx: vec![0],
        }
    }

    pub fn get_monitor_rect(&self) -> &position::Position {
        &self.monitor_rect
    }

    pub fn set_monitor_rect(&mut self, position: position::Position) {
        self.monitor_rect = position;
    }

    pub fn get_variants(&self) -> &variants_container::VariantsContainer<variant::Variant> {
        &self.variants
    }

    pub fn get_variants_mut(
        &mut self,
    ) -> &mut variants_container::VariantsContainer<variant::Variant> {
        &mut self.variants
    }

    pub fn default_variant_idx(&self) -> &Vec<usize> {
        &self.default_variant_idx
    }

    pub fn set_default_variant_idx(&mut self, idx: &[usize]) {
        self.default_variant_idx = Vec::from(idx);
    }

    pub fn update_all(&mut self, window_padding: i32, edge_padding: i32) {
        // for variant in self.variants.iter_mut() {
        //     variant.update(window_padding, edge_padding, &self.monitor_rect);
        // }
        self.variants.callback_all(|variant| {
            variant.update(window_padding, edge_padding, &self.monitor_rect);
        });
    }

    pub fn get_internal_positions(
        &mut self,
        variant_idx: &[usize],
        n: usize,
        window_padding: i32,
        edge_padding: i32,
    ) -> &Vec<position::Position> {
        self.variants
            .get_innermost_mut(variant_idx)
            .get_internal_positions(n, window_padding, edge_padding, &self.monitor_rect)
    }
}

impl TryFrom<user_layout::UserLayout<'_>> for Layout {
    type Error = serde_json::Error;

    fn try_from(value: user_layout::UserLayout) -> Result<Self, Self::Error> {
        let user_variants =
            variants_container::VariantsContainer::<user_layout::UserVariant>::from_raw_value(
                &value.variants,
            )?;
        Ok(Self {
            monitor_rect: position::Position::new(0, 0, value.w, value.h),
            variants: user_variants.map(variant::Variant::from),
            default_variant_idx: value.default_variant_idx,
        })
    }
}
