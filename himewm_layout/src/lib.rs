use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    pub fn other(&self) -> Self {
        match self {
            Direction::Horizontal => Direction::Vertical,
            Direction::Vertical => Direction::Horizontal,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SplitDirection {
    Horizontal(i32),
    Vertical(i32),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EndTilingBehaviour {
    Directional {
        direction: Direction,
        from_zones: Option<Vec<Zone>>,
        zone_idx: usize,
    },
    Repeating {
        splits: Vec<RepeatingSplit>,
        zone_idx: usize,
    },
}

impl EndTilingBehaviour {
    pub fn default_directional() -> Self {
        EndTilingBehaviour::Directional {
            direction: Direction::Vertical,
            from_zones: None,
            zone_idx: 0,
        }
    }

    pub fn default_repeating() -> Self {
        EndTilingBehaviour::Repeating {
            splits: Vec::new(),
            zone_idx: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RepeatingSplit {
    direction: Direction,
    split_ratio: f64,
    split_idx_offset: usize,
    swap: bool,
}

impl RepeatingSplit {
    pub fn new(
        direction: Direction,
        split_ratio: f64,
        split_idx_offset: usize,
        swap: bool,
    ) -> Self {
        RepeatingSplit {
            direction,
            split_ratio,
            split_idx_offset,
            swap,
        }
    }

    pub fn get_direction(&self) -> &Direction {
        &self.direction
    }

    pub fn get_split_ratio(&self) -> f64 {
        self.split_ratio
    }

    pub fn get_offset(&self) -> usize {
        self.split_idx_offset
    }

    pub fn get_swap(&self) -> bool {
        self.swap
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Zone {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl From<RECT> for Zone {
    fn from(value: RECT) -> Self {
        Zone {
            left: value.left,
            top: value.top,
            right: value.right,
            bottom: value.bottom,
        }
    }
}

impl Zone {
    fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Zone {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn w(&self) -> i32 {
        self.right - self.left
    }

    pub fn h(&self) -> i32 {
        self.bottom - self.top
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub cx: i32,
    pub cy: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variant {
    zones: Vec<Vec<Zone>>,
    manual_zones_until: usize,
    end_tiling_behaviour: EndTilingBehaviour,
    positions: Vec<Vec<Position>>,
}

impl Variant {
    pub fn new(w: i32, h: i32) -> Self {
        Variant {
            zones: vec![vec![Zone::new(0, 0, w, h)]],
            manual_zones_until: 1,
            end_tiling_behaviour: EndTilingBehaviour::default_directional(),
            positions: Vec::new(),
        }
    }

    pub fn get_zones(&self) -> &Vec<Vec<Zone>> {
        &self.zones
    }

    pub fn get_zones_mut(&mut self) -> &mut Vec<Vec<Zone>> {
        &mut self.zones
    }

    pub fn delete_zones(&mut self, i: usize) {
        self.zones.remove(i);
        self.manual_zones_until -= 1;
    }

    pub fn swap_zone_vectors(&mut self, i: usize, j: usize) {
        if i == j {
            return;
        }
        let first_idx = std::cmp::min(i, j);
        let second_idx = std::cmp::max(i, j);
        let (first_slice, second_slice) = self.zones.split_at_mut(second_idx);
        std::mem::swap(&mut first_slice[first_idx], &mut second_slice[0]);
    }

    pub fn manual_zones_until(&self) -> usize {
        self.manual_zones_until
    }

    pub fn get_positions_at(&self, i: usize) -> &Vec<Position> {
        &self.positions[i]
    }

    pub fn positions_len(&self) -> usize {
        self.positions.len()
    }

    pub fn update_from_zones(&mut self) {
        match &mut self.end_tiling_behaviour {
            EndTilingBehaviour::Directional {
                direction: _,
                from_zones,
                zone_idx: _,
            } if matches!(from_zones, None)
                && self.zones[self.zones.len() - 1].len() < self.zones.len() =>
            {
                *from_zones = self.zones.pop();
                self.manual_zones_until -= 1;
            }
            _ => (),
        }
    }

    pub fn using_from_zones(&self) -> bool {
        match &self.end_tiling_behaviour {
            EndTilingBehaviour::Directional {
                direction: _,
                from_zones,
                zone_idx: _,
            } => match from_zones {
                Some(_) => return true,
                None => return false,
            },
            EndTilingBehaviour::Repeating {
                splits: _,
                zone_idx: _,
            } => return false,
        }
    }

    pub fn update(&mut self, window_padding: i32, edge_padding: i32, monitor_rect: &Zone) {
        self.update_from_zones();
        self.positions = Vec::new();
        let mut len = 0;
        for zones in &self.zones {
            self.positions.push(Vec::new());
            len += 1;
            for zone in zones {
                let mut position = Position {
                    x: zone.left - 7 + window_padding,
                    y: zone.top + window_padding,
                    cx: zone.w() + 14 - 2 * window_padding,
                    cy: zone.h() + 7 - 2 * window_padding,
                };
                if zone.left == monitor_rect.left {
                    position.x = position.x - window_padding + edge_padding;
                    position.cx = position.cx + window_padding - edge_padding;
                }
                if zone.top == monitor_rect.top {
                    position.y = position.y - window_padding + edge_padding;
                    position.cy = position.cy + window_padding - edge_padding;
                }
                if zone.right == monitor_rect.right {
                    position.cx = position.cx + window_padding - edge_padding;
                }
                if zone.bottom == monitor_rect.bottom {
                    position.cy = position.cy + window_padding - edge_padding;
                }
                self.positions[len - 1].push(position);
            }
        }
    }

    pub fn get_end_tiling_behaviour(&self) -> &EndTilingBehaviour {
        &self.end_tiling_behaviour
    }

    pub fn set_end_tiling_behaviour(&mut self, behaviour: EndTilingBehaviour) {
        self.end_tiling_behaviour = behaviour;
    }

    pub fn get_end_zone_idx(&self) -> usize {
        match self.end_tiling_behaviour {
            EndTilingBehaviour::Directional {
                direction: _,
                from_zones: _,
                zone_idx,
            } => return zone_idx,
            EndTilingBehaviour::Repeating {
                splits: _,
                zone_idx,
            } => return zone_idx,
        }
    }

    pub fn set_end_zone_idx(&mut self, i: usize) {
        match &mut self.end_tiling_behaviour {
            EndTilingBehaviour::Directional {
                direction: _,
                from_zones: _,
                zone_idx,
            } => {
                *zone_idx = i;
            }
            EndTilingBehaviour::Repeating {
                splits: _,
                zone_idx,
            } => {
                *zone_idx = i;
            }
        }
    }

    pub fn get_end_tiling_direction(&self) -> Option<Direction> {
        match &self.end_tiling_behaviour {
            EndTilingBehaviour::Directional {
                direction,
                from_zones: _,
                zone_idx: _,
            } => return Some(direction.to_owned()),
            EndTilingBehaviour::Repeating {
                splits: _,
                zone_idx: _,
            } => return None,
        }
    }

    pub fn set_end_tiling_direction(&mut self, new_direction: Direction) {
        match &mut self.end_tiling_behaviour {
            EndTilingBehaviour::Directional {
                direction,
                from_zones: _,
                zone_idx: _,
            } => {
                *direction = new_direction;
            }
            EndTilingBehaviour::Repeating {
                splits: _,
                zone_idx: _,
            } => return,
        }
    }

    pub fn add_repeating_split(
        &mut self,
        direction: Direction,
        split_ratio: f64,
        split_idx_offset: usize,
        swap: bool,
    ) -> Option<&RepeatingSplit> {
        if let EndTilingBehaviour::Repeating {
            splits,
            zone_idx: _,
        } = &mut self.end_tiling_behaviour
        {
            let split = RepeatingSplit::new(direction, split_ratio, split_idx_offset, swap);
            splits.push(split);
            let ret = &splits[splits.len() - 1];
            return Some(ret);
        }
        return None;
    }

    pub fn delete_repeating_split(&mut self, idx: usize) {
        if let EndTilingBehaviour::Repeating {
            splits,
            zone_idx: _,
        } = &mut self.end_tiling_behaviour
        {
            splits.remove(idx);
            let max_offset = splits.len();
            for split in splits {
                if split.split_idx_offset > max_offset {
                    split.split_idx_offset = max_offset;
                }
            }
        }
    }

    pub fn set_repeating_split_direction(&mut self, idx: usize, direction: Direction) {
        if let EndTilingBehaviour::Repeating {
            splits,
            zone_idx: _,
        } = &mut self.end_tiling_behaviour
        {
            splits[idx].direction = direction;
        }
    }

    pub fn set_repeating_split_ratio(&mut self, idx: usize, val: f64) {
        if let EndTilingBehaviour::Repeating {
            splits,
            zone_idx: _,
        } = &mut self.end_tiling_behaviour
        {
            splits[idx].split_ratio = val;
        }
    }

    pub fn set_repeating_split_idx_offset(&mut self, idx: usize, val: usize) {
        if let EndTilingBehaviour::Repeating {
            splits,
            zone_idx: _,
        } = &mut self.end_tiling_behaviour
        {
            splits[idx].split_idx_offset = val;
        }
    }

    pub fn set_repeating_split_swap(&mut self, idx: usize, val: bool) {
        if let EndTilingBehaviour::Repeating {
            splits,
            zone_idx: _,
        } = &mut self.end_tiling_behaviour
        {
            splits[idx].swap = val;
        }
    }

    pub fn new_zone_vec(&mut self, w: i32, h: i32) {
        self.zones.push(vec![Zone::new(0, 0, w, h)]);
        self.manual_zones_until += 1;
    }

    pub fn clone_zone_vec(&mut self, i: usize) {
        self.zones.push(self.zones[i].clone());
        self.manual_zones_until += 1;
    }

    pub fn split(&mut self, i: usize, j: usize, direction: SplitDirection) {
        let zone = &mut self.zones[i][j];
        let new_zone;
        match direction {
            SplitDirection::Horizontal(at) => {
                if at - zone.left < zone.w() / 2 {
                    new_zone = Zone::new(zone.left, zone.top, at, zone.bottom);
                    zone.left = at;
                } else {
                    new_zone = Zone::new(at, zone.top, zone.right, zone.bottom);
                    zone.right = at;
                }
            }
            SplitDirection::Vertical(at) => {
                if at - zone.top < zone.h() / 2 {
                    new_zone = Zone::new(zone.left, zone.top, zone.right, at);
                    zone.top = at;
                } else {
                    new_zone = Zone::new(zone.left, at, zone.right, zone.bottom);
                    zone.bottom = at;
                }
            }
        }
        self.set_end_zone_idx(self.zones[i].len());
        self.zones[i].push(new_zone);
    }

    pub fn can_merge_zones(&self, i: usize, j: usize, k: usize) -> bool {
        if j == k {
            return false;
        }
        let first_zone = &self.zones[i][j];
        let second_zone = &self.zones[i][k];
        return (first_zone.left == second_zone.left
            && first_zone.right == second_zone.right
            && (first_zone.bottom == second_zone.top || first_zone.top == second_zone.bottom))
            || (first_zone.top == second_zone.top
                && first_zone.bottom == second_zone.bottom
                && (first_zone.right == second_zone.left || first_zone.left == second_zone.right));
    }

    pub fn merge_zones(&mut self, i: usize, j: usize, k: usize) {
        if j == k {
            return;
        }
        let first_idx = std::cmp::min(j, k);
        let second_idx = std::cmp::max(j, k);
        if self.zones[i][first_idx].left == self.zones[i][second_idx].left
            && self.zones[i][first_idx].right == self.zones[i][second_idx].right
        {
            if self.zones[i][first_idx].bottom == self.zones[i][second_idx].top {
                self.zones[i][first_idx].bottom = self.zones[i][second_idx].bottom;
            } else if self.zones[i][first_idx].top == self.zones[i][second_idx].bottom {
                self.zones[i][first_idx].top = self.zones[i][second_idx].top;
            } else {
                return;
            }
        } else if self.zones[i][first_idx].top == self.zones[i][second_idx].top
            && self.zones[i][first_idx].bottom == self.zones[i][second_idx].bottom
        {
            if self.zones[i][first_idx].right == self.zones[i][second_idx].left {
                self.zones[i][first_idx].right = self.zones[i][second_idx].right;
            } else if self.zones[i][first_idx].left == self.zones[i][second_idx].right {
                self.zones[i][first_idx].left = self.zones[i][second_idx].left;
            } else {
                return;
            }
        } else {
            return;
        }
        self.zones[i].remove(second_idx);
    }

    pub fn swap_zones(&mut self, i: usize, j: usize, k: usize) {
        if j == k {
            return;
        }
        let first_idx = std::cmp::min(j, k);
        let second_idx = std::cmp::max(j, k);
        let (first_slice, second_slice) = self.zones[i].split_at_mut(second_idx);
        std::mem::swap(&mut first_slice[first_idx], &mut second_slice[0]);
    }

    pub fn merge_and_split_zones(
        &mut self,
        i: usize,
        j: usize,
        k: usize,
        direction: SplitDirection,
    ) {
        let first_idx = std::cmp::min(j, k);
        let second_idx = std::cmp::max(j, k);
        self.merge_zones(i, j, k);
        self.split(i, first_idx, direction);
        let zone = self.zones[i].pop().unwrap();
        self.zones[i].insert(second_idx, zone);
    }

    pub fn extend(&mut self) {
        let end_zone_idx = self.get_end_zone_idx();
        let end_tiling_behaviour = self.end_tiling_behaviour.clone();
        match end_tiling_behaviour {
            EndTilingBehaviour::Directional {
                direction,
                from_zones,
                zone_idx,
            } => {
                let used_from_zones = match &from_zones {
                    Some(zones) => {
                        self.zones.push(zones.clone());
                        true
                    }
                    None => {
                        self.zones
                            .push(self.zones[self.manual_zones_until - 1].clone());
                        false
                    }
                };
                match direction {
                    Direction::Horizontal => {
                        let offset = (self.zones[self.zones.len() - 1][zone_idx].w())
                            / (self.zones.len() - self.zones[self.zones.len() - 1].len() + 1)
                                as i32;
                        while self.zones[self.zones.len() - 1].len() < self.zones.len() {
                            self.split(
                                self.zones.len() - 1,
                                zone_idx,
                                SplitDirection::Horizontal(
                                    self.zones[self.zones.len() - 1][zone_idx].left + offset,
                                ),
                            );
                            self.set_end_zone_idx(end_zone_idx);
                        }
                    }
                    Direction::Vertical => {
                        let offset = (self.zones[self.zones.len() - 1][zone_idx].h())
                            / (self.zones.len() - self.zones[self.zones.len() - 1].len() + 1)
                                as i32;
                        while self.zones[self.zones.len() - 1].len() < self.zones.len() {
                            self.split(
                                self.zones.len() - 1,
                                zone_idx,
                                SplitDirection::Vertical(
                                    self.zones[self.zones.len() - 1][zone_idx].top + offset,
                                ),
                            );
                            self.set_end_zone_idx(end_zone_idx);
                        }
                    }
                }
                if used_from_zones {
                    for i in (from_zones.unwrap().len()..(self.zones.len() - 1)).rev() {
                        self.swap_zones(self.zones.len() - 1, zone_idx, i);
                    }
                } else {
                    for i in (self.manual_zones_until..(self.zones.len() - 1)).rev() {
                        self.swap_zones(self.zones.len() - 1, zone_idx, i);
                    }
                }
            }
            EndTilingBehaviour::Repeating { splits, zone_idx } => {
                let repeating_split_idx =
                    (self.zones.len() - self.manual_zones_until) % splits.len();
                let split = &splits[repeating_split_idx];
                self.zones.push(self.zones[self.zones.len() - 1].clone());
                let split_idx = if self.zones.len() - 1 == self.manual_zones_until {
                    zone_idx
                } else if repeating_split_idx == 0 {
                    self.zones.len() - 1 - splits.len() + split.split_idx_offset
                } else {
                    self.zones.len() - 1 - repeating_split_idx + split.split_idx_offset
                };
                let at;
                match split.direction {
                    Direction::Horizontal => {
                        at = self.zones[self.zones.len() - 1][split_idx].left
                            + (split.split_ratio
                                * (self.zones[self.zones.len() - 1][split_idx].w() as f64))
                                .round() as i32;
                        self.split(
                            self.zones.len() - 1,
                            split_idx,
                            SplitDirection::Horizontal(at),
                        );
                    }
                    Direction::Vertical => {
                        at = self.zones[self.zones.len() - 1][split_idx].top
                            + (split.split_ratio
                                * (self.zones[self.zones.len() - 1][split_idx].h() as f64))
                                .round() as i32;
                        self.split(
                            self.zones.len() - 1,
                            split_idx,
                            SplitDirection::Vertical(at),
                        );
                    }
                }
                self.set_end_zone_idx(end_zone_idx);
                if split.swap {
                    self.swap_zones(
                        self.zones.len() - 1,
                        split_idx,
                        self.zones[self.zones.len() - 1].len() - 1,
                    );
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Layout {
    monitor_rect: Zone,
    variants: Vec<Variant>,
    default_variant_idx: usize,
}

impl Layout {
    pub fn new(w: i32, h: i32) -> Self {
        Layout {
            monitor_rect: Zone::new(0, 0, w, h),
            variants: vec![Variant::new(w, h)],
            default_variant_idx: 0,
        }
    }

    pub fn get_monitor_rect(&self) -> &Zone {
        &self.monitor_rect
    }

    pub fn set_monitor_rect(&mut self, zone: Zone) {
        self.monitor_rect = zone;
    }

    pub fn get_variants(&self) -> &Vec<Variant> {
        &self.variants
    }

    pub fn get_variants_mut(&mut self) -> &mut Vec<Variant> {
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

    pub fn update_all(&mut self, window_padding: i32, edge_padding: i32, monitor_rect: &Zone) {
        for variant in self.variants.iter_mut() {
            variant.update(window_padding, edge_padding, monitor_rect);
        }
    }

    pub fn new_variant(&mut self) {
        self.variants.push(Variant::new(
            self.monitor_rect.right,
            self.monitor_rect.bottom,
        ));
    }

    pub fn clone_variant(&mut self, idx: usize) {
        self.variants.push(self.variants[idx].clone());
    }

    pub fn swap_variants(&mut self, i: usize, j: usize) {
        if i == j {
            return;
        }
        if self.default_variant_idx == i {
            self.default_variant_idx = j;
        } else if self.default_variant_idx == j {
            self.default_variant_idx = i;
        }
        let first_idx = std::cmp::min(i, j);
        let second_idx = std::cmp::max(i, j);
        let (first_slice, second_slice) = self.variants.split_at_mut(second_idx);
        std::mem::swap(&mut first_slice[first_idx], &mut second_slice[0]);
    }

    pub fn delete_variant(&mut self, idx: usize) {
        self.variants.remove(idx);
        if idx == self.default_variant_idx {
            self.default_variant_idx = 0;
        } else if idx < self.default_variant_idx {
            self.default_variant_idx -= 1;
        }
    }
}
