use crate::common;
use serde::{Deserialize, Serialize};

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
        from: Option<Vec<common::Position>>,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variant {
    positions: Vec<Vec<common::Position>>,
    internal_positions: Vec<Vec<common::Position>>,
    manual_positions_until: usize,
    end_tiling_behaviour: EndTilingBehaviour,
}

impl Variant {
    pub fn new(w: i32, h: i32) -> Self {
        Self {
            positions: vec![vec![common::Position::new(0, 0, w, h)]],
            internal_positions: Vec::new(),
            manual_positions_until: 1,
            end_tiling_behaviour: EndTilingBehaviour::default_directional(),
        }
    }

    pub fn get_positions(&self) -> &Vec<Vec<common::Position>> {
        &self.positions
    }

    pub fn get_positions_mut(&mut self) -> &mut Vec<Vec<common::Position>> {
        &mut self.positions
    }

    pub fn update(
        &mut self,
        window_padding: i32,
        edge_padding: i32,
        monitor_rect: &common::Position,
    ) {
        self.internal_positions = Vec::new();
        let mut len = 0;
        for positions in &self.positions {
            self.internal_positions.push(Vec::new());
            len += 1;
            for position in positions {
                let mut internal_position = common::Position::new(
                    position.x() - 7 + window_padding,
                    position.y() + window_padding,
                    position.w() + 14 - 2 * window_padding,
                    position.h() + 7 - 2 * window_padding,
                );
                if position.x() == monitor_rect.x() {
                    internal_position.set_x(internal_position.x() - window_padding + edge_padding);
                    internal_position.set_w(internal_position.w() + window_padding - edge_padding);
                }
                if position.y() == monitor_rect.y() {
                    internal_position.set_y(internal_position.y() - window_padding + edge_padding);
                    internal_position.set_h(internal_position.h() + window_padding - edge_padding);
                }
                if position.x() + position.w() == monitor_rect.x() + monitor_rect.w() {
                    internal_position.set_w(internal_position.w() + window_padding - edge_padding);
                }
                if position.y() + position.h() == monitor_rect.y() + monitor_rect.h() {
                    internal_position.set_h(internal_position.h() + window_padding - edge_padding);
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
                if w < position.w() / 2 {
                    new_position =
                        common::Position::new(position.x(), position.y(), w, position.h());
                    position.set_x(position.x() + w);
                } else {
                    new_position =
                        common::Position::new(position.x() + w, position.y(), w, position.h());
                }
                position.set_w(position.w() - w);
            }
            SplitDirection::Vertical(h) => {
                if h < position.h() / 2 {
                    new_position =
                        common::Position::new(position.x(), position.y(), position.w(), h);
                    position.set_y(position.y() + h);
                } else {
                    new_position =
                        common::Position::new(position.x(), position.y() + h, position.w(), h);
                }
                position.set_h(position.h() - h);
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
                        let w = (self.positions[self.positions.len() - 1][position_idx].w())
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
                            * (self.positions[self.positions.len() - 1][split_idx].w() as f64))
                            .round() as i32;
                        self.split(
                            self.positions.len() - 1,
                            split_idx,
                            SplitDirection::Horizontal(w),
                        );
                    }
                    Direction::Vertical => {
                        let h = (split.split_ratio
                            * (self.positions[self.positions.len() - 1][split_idx].h() as f64))
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
        monitor_rect: &common::Position,
    ) -> &Vec<common::Position> {
        while self.positions.len() < n {
            self.extend();
        }
        self.update(window_padding, edge_padding, monitor_rect);
        return &self.internal_positions[n - 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::*;
    #[test]
    fn extending() {
        let mut layout = Layout::new(1920, 1200);
        let variant = &mut layout.get_variants_mut()[0];
        variant.end_tiling_behaviour = EndTilingBehaviour {
            position_idx: 1,
            behaviour: EndBehaviourType::Directional {
                direction: Direction::Vertical,
                from: None,
            },
        };
        variant.positions.push(vec![
            common::Position::new(0, 0, 960, 1200),
            common::Position::new(960, 0, 960, 1200),
        ]);
        variant.manual_positions_until = 2;
        variant.extend();
        let last_positions = &variant.positions[variant.positions.len() - 1];
        assert_eq!(last_positions[0], common::Position::new(0, 0, 960, 1200));
        assert_eq!(last_positions[1], common::Position::new(960, 0, 960, 600));
        assert_eq!(last_positions[2], common::Position::new(960, 600, 960, 600));
    }
}
