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
pub enum EndBehaviourType {
    Directional {
        direction: Direction,
        from: Option<Vec<Position>>,
    },
    Repeating {
        splits: Vec<RepeatingSplit>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndTilingBehaviour {
    position_idx: usize,
    behaviour: EndBehaviourType,
}

impl EndTilingBehaviour {
    pub fn default_directional() -> Self {
        Self {
            position_idx: 0,
            behaviour: EndBehaviourType::Directional {
                direction: Direction::Vertical,
                from: None,
            },
        }
    }

    pub fn default_repeating() -> Self {
        Self {
            position_idx: 0,
            behaviour: EndBehaviourType::Repeating { splits: Vec::new() },
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
pub struct Position(i32, i32, i32, i32);

impl From<RECT> for Position {
    fn from(value: RECT) -> Self {
        Self(
            value.left,
            value.top,
            value.right - value.left,
            value.bottom - value.top,
        )
    }
}

impl From<(i32, i32, i32, i32)> for Position {
    fn from(value: (i32, i32, i32, i32)) -> Self {
        Self(value.0, value.1, value.2, value.3)
    }
}

impl Position {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self(x, y, w, h)
    }

    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn set_x(&mut self, val: i32) {
        self.0 = val;
    }

    pub fn y(&self) -> i32 {
        self.1
    }

    pub fn set_y(&mut self, val: i32) {
        self.1 = val;
    }

    pub fn w(&self) -> i32 {
        self.2
    }

    pub fn set_w(&mut self, val: i32) {
        self.2 = val;
    }

    pub fn h(&self) -> i32 {
        self.3
    }

    pub fn set_h(&mut self, val: i32) {
        self.3 = val;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InternalVariant {
    positions: Vec<Vec<Position>>,
    internal_positions: Vec<Vec<Position>>,
    manual_positions_until: usize,
    end_tiling_behaviour: EndTilingBehaviour,
}

impl InternalVariant {
    pub fn new(w: i32, h: i32) -> Self {
        Self {
            positions: vec![vec![Position::new(0, 0, w, h)]],
            internal_positions: Vec::new(),
            manual_positions_until: 1,
            end_tiling_behaviour: EndTilingBehaviour::default_directional(),
        }
    }

    pub fn get_positions(&self) -> &Vec<Vec<Position>> {
        &self.positions
    }

    pub fn get_positions_mut(&mut self) -> &mut Vec<Vec<Position>> {
        &mut self.positions
    }

    pub fn update(&mut self, window_padding: i32, edge_padding: i32, monitor_rect: &Position) {
        self.internal_positions = Vec::new();
        let mut len = 0;
        for positions in &self.positions {
            self.internal_positions.push(Vec::new());
            len += 1;
            for position in positions {
                let mut internal_position = Position(
                    position.0 - 7 + window_padding,
                    position.1 + window_padding,
                    position.2 + 14 - 2 * window_padding,
                    position.3 + 7 - 2 * window_padding,
                );
                if position.0 == monitor_rect.0 {
                    internal_position.0 = internal_position.0 - window_padding + edge_padding;
                    internal_position.2 = internal_position.2 + window_padding - edge_padding;
                }
                if position.1 == monitor_rect.1 {
                    internal_position.1 = internal_position.1 - window_padding + edge_padding;
                    internal_position.3 = internal_position.3 + window_padding - edge_padding;
                }
                if position.0 + position.2 == monitor_rect.0 + monitor_rect.2 {
                    internal_position.2 = internal_position.2 + window_padding - edge_padding;
                }
                if position.1 + position.3 == monitor_rect.1 + monitor_rect.3 {
                    internal_position.3 = internal_position.3 + window_padding - edge_padding;
                }
                self.internal_positions[len - 1].push(internal_position);
            }
        }
    }

    pub fn split(&mut self, i: usize, j: usize, direction: SplitDirection) {
        let position = &mut self.positions[i][j];
        let new_position;
        match direction {
            SplitDirection::Horizontal(w) => {
                if w < position.2 / 2 {
                    new_position = Position::new(position.0, position.1, w, position.3);
                    position.0 += w;
                } else {
                    new_position = Position::new(position.0 + w, position.1, w, position.3);
                }
                position.2 -= w;
            }
            SplitDirection::Vertical(h) => {
                if h < position.3 / 2 {
                    new_position = Position::new(position.0, position.1, position.2, h);
                    position.1 += h;
                } else {
                    new_position = Position::new(position.0, position.1 + h, position.2, h);
                }
                position.3 -= h;
            }
        }
        self.positions[i].push(new_position);
    }

    pub fn extend(&mut self) {
        let position_idx = self.end_tiling_behaviour.position_idx;
        match self.end_tiling_behaviour.behaviour.to_owned() {
            EndBehaviourType::Directional { direction, from } => {
                let used_from = match &from {
                    Some(positions) => {
                        self.positions.push(positions.clone());
                        true
                    }
                    None => {
                        self.positions
                            .push(self.positions[self.manual_positions_until - 1].clone());
                        false
                    }
                };
                match direction {
                    Direction::Horizontal => {
                        let w = (self.positions[self.positions.len() - 1][position_idx].2)
                            / (self.positions.len()
                                - self.positions[self.positions.len() - 1].len()
                                + 1) as i32;
                        while self.positions[self.positions.len() - 1].len() < self.positions.len()
                        {
                            self.split(
                                self.positions.len() - 1,
                                position_idx,
                                SplitDirection::Horizontal(w),
                            );
                        }
                    }
                    Direction::Vertical => {
                        let h = (self.positions[self.positions.len() - 1][position_idx].h())
                            / (self.positions.len()
                                - self.positions[self.positions.len() - 1].len()
                                + 1) as i32;
                        while self.positions[self.positions.len() - 1].len() < self.positions.len()
                        {
                            self.split(
                                self.positions.len() - 1,
                                position_idx,
                                SplitDirection::Vertical(h),
                            );
                        }
                    }
                }
                let last_idx = self.positions.len() - 1;
                if used_from {
                    for i in (from.unwrap().len()..(self.positions.len() - 1)).rev() {
                        self.positions[last_idx].swap(position_idx, i);
                    }
                } else {
                    for i in (self.manual_positions_until..(self.positions.len() - 1)).rev() {
                        self.positions[last_idx].swap(position_idx, i);
                    }
                }
            }
            EndBehaviourType::Repeating { splits } => {
                let repeating_split_idx =
                    (self.positions.len() - self.manual_positions_until) % splits.len();
                let split = &splits[repeating_split_idx];
                let split_idx = if self.positions.len() == self.manual_positions_until {
                    position_idx
                } else if repeating_split_idx == 0 {
                    self.positions.len() - 1 - splits.len() + split.split_idx_offset
                } else {
                    self.positions.len() - 1 - repeating_split_idx + split.split_idx_offset
                };
                self.positions
                    .push(self.positions[self.positions.len() - 1].clone());
                match split.direction {
                    Direction::Horizontal => {
                        let w = (split.split_ratio
                            * (self.positions[self.positions.len() - 1][split_idx].2 as f64))
                            .round() as i32;
                        self.split(
                            self.positions.len() - 1,
                            split_idx,
                            SplitDirection::Horizontal(w),
                        );
                    }
                    Direction::Vertical => {
                        let h = (split.split_ratio
                            * (self.positions[self.positions.len() - 1][split_idx].3 as f64))
                            .round() as i32;
                        self.split(
                            self.positions.len() - 1,
                            split_idx,
                            SplitDirection::Vertical(h),
                        );
                    }
                }
                if split.swap {
                    let last_idx = self.positions.len() - 1;
                    let swap_idx = self.positions[last_idx].len() - 1;
                    self.positions[last_idx].swap(split_idx, swap_idx);
                }
            }
        }
    }

    pub fn get_internal_positions(
        &mut self,
        n: usize,
        window_padding: i32,
        edge_padding: i32,
        monitor_rect: &Position,
    ) -> &Vec<Position> {
        while self.positions.len() < n {
            self.extend();
        }
        self.update(window_padding, edge_padding, monitor_rect);
        return &self.internal_positions[n - 1];
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Layout {
    monitor_rect: Position,
    variants: Vec<InternalVariant>,
    default_variant_idx: usize,
}

impl Layout {
    pub fn new(w: i32, h: i32) -> Self {
        Layout {
            monitor_rect: Position::new(0, 0, w, h),
            variants: vec![InternalVariant::new(w, h)],
            default_variant_idx: 0,
        }
    }

    pub fn get_monitor_rect(&self) -> &Position {
        &self.monitor_rect
    }

    pub fn set_monitor_rect(&mut self, position: Position) {
        self.monitor_rect = position;
    }

    pub fn get_variants(&self) -> &Vec<InternalVariant> {
        &self.variants
    }

    pub fn get_variants_mut(&mut self) -> &mut Vec<InternalVariant> {
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
    ) -> &Vec<Position> {
        self.variants[variant_idx].get_internal_positions(
            n,
            window_padding,
            edge_padding,
            &self.monitor_rect,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extending() {
        let mut layout = Layout::new(1920, 1200);
        let variant = &mut layout.variants[0];
        variant.end_tiling_behaviour = EndTilingBehaviour {
            position_idx: 1,
            behaviour: EndBehaviourType::Directional {
                direction: Direction::Vertical,
                from: None,
            },
        };
        variant
            .positions
            .push(vec![Position(0, 0, 960, 1200), Position(960, 0, 960, 1200)]);
        variant.manual_positions_until = 2;
        variant.extend();
        let last_positions = &variant.positions[variant.positions.len() - 1];
        assert_eq!(last_positions[0], Position(0, 0, 960, 1200));
        assert_eq!(last_positions[1], Position(960, 0, 960, 600));
        assert_eq!(last_positions[2], Position(960, 600, 960, 600));
    }
}
