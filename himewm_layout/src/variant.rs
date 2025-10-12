use crate::{position, user_layout};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn other(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EndBehaviourType {
    Directional { direction: Direction },
    Repeating { splits: Vec<RepeatingSplit> },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndBehaviour {
    from: Option<Vec<position::Position>>,
    position_idx: usize,
    behaviour: EndBehaviourType,
}

impl EndBehaviour {
    pub fn from(&self) -> &Option<Vec<position::Position>> {
        &self.from
    }

    pub fn from_mut(&mut self) -> &mut Option<Vec<position::Position>> {
        &mut self.from
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RepeatingSplit {
    direction: Direction,
    ratio: f64,
    offset: usize,
}

impl RepeatingSplit {
    pub fn new(direction: Direction, ratio: f64, offset: usize) -> Self {
        RepeatingSplit {
            direction,
            ratio,
            offset,
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
            end_behaviour: value.end_behaviour,
        };
    }
}

impl Variant {
    pub fn positions(&self) -> &Vec<Vec<position::Position>> {
        &self.positions
    }

    pub fn positions_mut(&mut self) -> &mut Vec<Vec<position::Position>> {
        &mut self.positions
    }

    pub fn end_behaviour(&self) -> &EndBehaviour {
        &self.end_behaviour
    }

    pub fn end_behaviour_mut(&mut self) -> &mut EndBehaviour {
        &mut self.end_behaviour
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
                    position.x() + window_padding,
                    position.y() + window_padding,
                    position.w() - 2 * window_padding,
                    position.h() - 2 * window_padding,
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

    pub fn split(&mut self, i: usize, j: usize, direction: Direction, at: i32) {
        let position = &mut self.positions[i][j];
        let new_position;
        match direction {
            Direction::Up => {
                new_position =
                    position::Position::new(position.x(), position.y(), position.w(), at);
                position.set_y(position.y() + at);
                position.set_h(position.h() - at);
            }
            Direction::Down => {
                new_position = position::Position::new(
                    position.x(),
                    position.y() + at,
                    position.w(),
                    position.h() - at,
                );
                position.set_h(at);
            }
            Direction::Left => {
                new_position =
                    position::Position::new(position.x(), position.y(), at, position.h());
                position.set_x(position.x() + at);
                position.set_w(position.w() - at);
            }
            Direction::Right => {
                new_position = position::Position::new(
                    position.x() + at,
                    position.y(),
                    position.w() - at,
                    position.h(),
                );
                position.set_w(at);
            }
        }
        self.positions[i].push(new_position);
    }

    fn apply_repeating_split(&mut self, split: &RepeatingSplit, split_idx: usize) {
        let at = match split.direction {
            Direction::Up | Direction::Down => (split.ratio
                * (self.positions[self.positions.len() - 1][split_idx].h() as f64))
                .round() as i32,
            Direction::Left | Direction::Right => (split.ratio
                * (self.positions[self.positions.len() - 1][split_idx].w() as f64))
                .round() as i32,
        };
        self.split(
            self.positions.len() - 1,
            split_idx,
            split.direction.to_owned(),
            at,
        );
    }

    pub fn extend(&mut self) {
        match self.end_behaviour.behaviour.to_owned() {
            EndBehaviourType::Directional { direction } => {
                match &self.end_behaviour.from {
                    Some(positions) => {
                        self.positions.push(positions.clone());
                    }
                    None => {
                        self.positions
                            .push(self.positions[self.manual_positions_until - 1].clone());
                    }
                };
                let mut first_iteration = true;
                let last_idx = self.positions.len() - 1;
                match direction {
                    Direction::Up => {
                        let h = (self.positions[last_idx][self.end_behaviour.position_idx].h()
                            as f64
                            / (self.positions.len() - self.positions[last_idx].len() + 1) as f64)
                            .round() as i32;
                        while self.positions[last_idx].len() < self.positions.len() {
                            let split_idx = if first_iteration {
                                self.end_behaviour.position_idx
                            } else {
                                self.positions[last_idx].len() - 1
                            };
                            let at = self.positions[last_idx][split_idx].h() - h;
                            self.split(last_idx, split_idx, Direction::Up, at);
                            first_iteration = false;
                        }
                    }
                    Direction::Down => {
                        let h = (self.positions[last_idx][self.end_behaviour.position_idx].h()
                            as f64
                            / (self.positions.len() - self.positions[last_idx].len() + 1) as f64)
                            .round() as i32;
                        while self.positions[last_idx].len() < self.positions.len() {
                            let split_idx = if first_iteration {
                                self.end_behaviour.position_idx
                            } else {
                                self.positions[last_idx].len() - 1
                            };
                            self.split(last_idx, split_idx, Direction::Down, h);
                            first_iteration = false;
                        }
                    }
                    Direction::Left => {
                        let w = (self.positions[last_idx][self.end_behaviour.position_idx].w()
                            as f64
                            / (self.positions.len() - self.positions[last_idx].len() + 1) as f64)
                            .round() as i32;
                        while self.positions[last_idx].len() < self.positions.len() {
                            let split_idx = if first_iteration {
                                self.end_behaviour.position_idx
                            } else {
                                self.positions[last_idx].len() - 1
                            };
                            let at = self.positions[last_idx][split_idx].w() - w;
                            self.split(last_idx, split_idx, Direction::Left, at);
                            first_iteration = false;
                        }
                    }
                    Direction::Right => {
                        let w = (self.positions[last_idx][self.end_behaviour.position_idx].w()
                            as f64
                            / (self.positions.len() - self.positions[last_idx].len() + 1) as f64)
                            .round() as i32;
                        while self.positions[last_idx].len() < self.positions.len() {
                            let split_idx = if first_iteration {
                                self.end_behaviour.position_idx
                            } else {
                                self.positions[last_idx].len() - 1
                            };
                            self.split(last_idx, split_idx, Direction::Right, w);
                            first_iteration = false;
                        }
                    }
                }
            }
            EndBehaviourType::Repeating { splits } => match &self.end_behaviour.from {
                Some(positions) if self.positions.len() == self.manual_positions_until => {
                    self.positions.push(positions.clone());
                    let last_idx = self.positions.len() - 1;
                    let mut repeating_split_idx = 0;
                    let mut split_idx = self.end_behaviour.position_idx;
                    let mut split = &splits[repeating_split_idx];
                    while self.positions[last_idx].len() < self.positions.len() {
                        self.apply_repeating_split(split, split_idx);
                        repeating_split_idx = if repeating_split_idx == splits.len() - 1 {
                            0
                        } else {
                            repeating_split_idx + 1
                        };
                        split = &splits[repeating_split_idx];
                        split_idx = if repeating_split_idx == 0 {
                            self.positions[last_idx].len() - 1 - splits.len() + split.offset
                        } else {
                            self.positions[last_idx].len() - 1 - repeating_split_idx + split.offset
                        };
                    }
                }
                _ => {
                    let repeating_split_idx =
                        (self.positions.len() - self.manual_positions_until) % splits.len();
                    let split = &splits[repeating_split_idx];
                    let split_idx = if self.positions.len() == self.manual_positions_until {
                        self.end_behaviour.position_idx
                    } else if repeating_split_idx == 0 {
                        self.positions.len() - 1 - splits.len() + split.offset
                    } else {
                        self.positions.len() - 1 - repeating_split_idx + split.offset
                    };
                    self.positions
                        .push(self.positions[self.positions.len() - 1].clone());
                    self.apply_repeating_split(split, split_idx);
                }
            },
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
