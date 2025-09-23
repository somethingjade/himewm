use crate::{position, user_layout};
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
        from: Option<Vec<position::Position>>,
    },
    Repeating {
        splits: Vec<RepeatingSplit>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndBehaviour {
    position_idx: usize,
    behaviour: EndBehaviourType,
}

impl EndBehaviour {
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
    ratio: f64,
    offset: usize,
    swap: bool,
}

impl RepeatingSplit {
    pub fn new(
        direction: Direction,
        ratio: f64,
        offset: usize,
        swap: bool,
    ) -> Self {
        RepeatingSplit {
            direction,
            ratio,
            offset,
            swap,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variant {
    positions: Vec<Vec<position::Position>>,
    internal_positions: Vec<Vec<position::Position>>,
    manual_positions_until: usize,
    end_behaviour: EndBehaviour,
}

impl From<user_layout::UserVariant> for Variant {
    fn from(value: user_layout::UserVariant) -> Self {
        let positions_len = value.positions.len();

        return Self {
            positions: value.positions,
            internal_positions: Vec::new(),
            manual_positions_until: positions_len,
            end_behaviour: value.end_behaviour
        };
    }
}

impl Variant {
    pub fn new(w: i32, h: i32) -> Self {
        Self {
            positions: vec![vec![position::Position::new(0, 0, w, h)]],
            internal_positions: Vec::new(),
            manual_positions_until: 1,
            end_behaviour: EndBehaviour::default_directional(),
        }
    }

    pub fn get_positions(&self) -> &Vec<Vec<position::Position>> {
        &self.positions
    }

    pub fn get_positions_mut(&mut self) -> &mut Vec<Vec<position::Position>> {
        &mut self.positions
    }

    pub fn update(
        &mut self,
        window_padding: i32,
        edge_padding: i32,
        monitor_rect: &position::Position,
    ) {
        self.internal_positions = Vec::new();
        let mut len = 0;
        for positions in &self.positions {
            self.internal_positions.push(Vec::new());
            len += 1;
            for position in positions {
                let mut internal_position = position::Position::new(
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
                        position::Position::new(position.x(), position.y(), w, position.h());
                    position.set_x(position.x() + w);
                } else {
                    new_position =
                        position::Position::new(position.x() + w, position.y(), w, position.h());
                }
                position.set_w(position.w() - w);
            }
            SplitDirection::Vertical(h) => {
                if h < position.h() / 2 {
                    new_position =
                        position::Position::new(position.x(), position.y(), position.w(), h);
                    position.set_y(position.y() + h);
                } else {
                    new_position =
                        position::Position::new(position.x(), position.y() + h, position.w(), h);
                }
                position.set_h(position.h() - h);
            }
        }
        self.positions[i].push(new_position);
    }

    pub fn extend(&mut self) {
        let position_idx = self.end_behaviour.position_idx;
        match self.end_behaviour.behaviour.to_owned() {
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
                    self.positions.len() - 1 - splits.len() + split.offset
                } else {
                    self.positions.len() - 1 - repeating_split_idx + split.offset
                };
                self.positions
                    .push(self.positions[self.positions.len() - 1].clone());
                match split.direction {
                    Direction::Horizontal => {
                        let w = (split.ratio
                            * (self.positions[self.positions.len() - 1][split_idx].w() as f64))
                            .round() as i32;
                        self.split(
                            self.positions.len() - 1,
                            split_idx,
                            SplitDirection::Horizontal(w),
                        );
                    }
                    Direction::Vertical => {
                        let h = (split.ratio
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
        monitor_rect: &position::Position,
    ) -> &Vec<position::Position> {
        while self.positions.len() < n {
            self.extend();
        }
        self.update(window_padding, edge_padding, monitor_rect);
        return &self.internal_positions[n - 1];
    }
}
