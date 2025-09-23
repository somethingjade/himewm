use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

pub enum VariantsContainerReturn<'a, T> {
    Container(&'a VariantsContainer<T>),
    Variant(&'a T),
}

pub enum VariantsContainerReturnMut<'a, T> {
    Container(&'a mut VariantsContainer<T>),
    Variant(&'a mut T),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum VariantsContainer<T> {
    Container(Vec<VariantsContainer<T>>),
    Variants(Vec<T>),
}

impl<T> VariantsContainer<T> {
    pub fn from_raw_value(raw: &RawValue) -> serde_json::Result<Self>
    where
        for<'a> T: Deserialize<'a>,
    {
        let raw_str = raw.get();
        let normalized_string = raw_str.replace([' ', '\n', '\t'], "");
        let data = normalized_string.as_bytes();
        let mut input_string = String::new();
        let mut inner_array_count = None;
        for i in 0..data.len() {
            match data[i] {
                b'[' if i + 1 < data.len() => match inner_array_count {
                    Some(count) => {
                        inner_array_count = Some(count + 1);
                        input_string.push('[');
                    }
                    None => {
                        if data[i + 1] == b'[' {
                            input_string += r#"{"Container":["#;
                        } else {
                            input_string += r#"{"Variants":["#;
                            inner_array_count = Some(0);
                        }
                    }
                },
                b']' => match inner_array_count {
                    Some(count) if count > 0 => {
                        input_string.push(']');
                        inner_array_count = Some(count - 1);
                    }
                    _ => {
                        input_string += "]}";
                        inner_array_count = None;
                    }
                },
                _ => {
                    input_string.push(data[i] as char);
                }
            }
        }
        return serde_json::from_str(&input_string);
    }

    pub fn len(&self) -> usize {
        match self {
            VariantsContainer::Container(inner) => {
                return inner.len();
            }
            VariantsContainer::Variants(inner) => {
                return inner.len();
            }
        }
    }

    pub fn get(&self, idx: &[usize]) -> VariantsContainerReturn<T> {
        let mut current = VariantsContainerReturn::Container(self);
        for i in idx {
            let current_layer = match current {
                VariantsContainerReturn::Container(container) if container.len() > 0 => container,
                _ => {
                    break;
                }
            };
            let current_idx = if *i > current_layer.len() - 1 {
                current_layer.len() - 1
            } else {
                *i
            };
            match current_layer {
                VariantsContainer::Container(inner) => {
                    current = VariantsContainerReturn::Container(&inner[current_idx]);
                }
                VariantsContainer::Variants(inner) => {
                    current = VariantsContainerReturn::Variant(&inner[current_idx]);
                }
            }
        }
        return current;
    }

    pub fn get_mut(&mut self, idx: &[usize]) -> VariantsContainerReturnMut<T> {
        let mut current = VariantsContainerReturnMut::Container(self);
        for i in idx {
            let current_layer = match current {
                VariantsContainerReturnMut::Container(container) if container.len() > 0 => {
                    container
                }
                _ => {
                    break;
                }
            };
            let current_idx = if *i > current_layer.len() - 1 {
                current_layer.len() - 1
            } else {
                *i
            };
            match current_layer {
                VariantsContainer::Container(inner) => {
                    current = VariantsContainerReturnMut::Container(&mut inner[current_idx]);
                }
                VariantsContainer::Variants(inner) => {
                    current = VariantsContainerReturnMut::Variant(&mut inner[current_idx]);
                }
            }
        }
        return current;
    }

    pub fn get_innermost(&self, idx: &[usize]) -> &T {
        match self.get(idx) {
            VariantsContainerReturn::Container(container) => {
                let inner_idx = [0];
                let mut current_layer = container;
                loop {
                    match current_layer.get(&inner_idx) {
                        VariantsContainerReturn::Container(next_container) => {
                            current_layer = next_container;
                        }
                        VariantsContainerReturn::Variant(variant) => {
                            return variant;
                        }
                    }
                }
            }
            VariantsContainerReturn::Variant(variant) => {
                return variant;
            }
        }
    }

    pub fn get_innermost_mut(&mut self, idx: &[usize]) -> &mut T {
        match self.get_mut(idx) {
            VariantsContainerReturnMut::Container(container) => {
                let inner_idx = [0];
                let mut current_layer = container;
                loop {
                    match current_layer.get_mut(&inner_idx) {
                        VariantsContainerReturnMut::Container(next_container) => {
                            current_layer = next_container;
                        }
                        VariantsContainerReturnMut::Variant(variant) => {
                            return variant;
                        }
                    }
                }
            }
            VariantsContainerReturnMut::Variant(variant) => {
                return variant;
            }
        }
    }

    pub fn callback_all<F: FnMut(&mut T)>(&mut self, mut cb: F) {
        let mut stack = Vec::new();
        stack.push(vec![0]);
        while !stack.is_empty() {
            let current_idx = stack.pop().unwrap();
            match self.get_mut(&current_idx) {
                VariantsContainerReturnMut::Container(container) => {
                    for i in 0..container.len() {
                        stack.push([current_idx.as_slice(), &[i]].concat());
                    }
                }
                VariantsContainerReturnMut::Variant(variant) => {
                    cb(variant);
                }
            }
        }
    }
}

impl<T: Clone> VariantsContainer<T> {
    pub fn map<U, F: Fn(T) -> U>(&self, cb: F) -> VariantsContainer<U> {
        let mut ret = match self {
            VariantsContainer::Container(_) => VariantsContainer::Container(Vec::new()),
            VariantsContainer::Variants(inner) => {
                return VariantsContainer::Variants(
                    inner.iter().map(|variant| cb(variant.to_owned())).collect(),
                );
            }
        };
        let mut stack = vec![vec![]];
        while !stack.is_empty() {
            let current_idx = stack.pop().unwrap();
            if let VariantsContainerReturn::Container(container) = self.get(&current_idx) {
                match container {
                    VariantsContainer::Container(inner) => {
                        if let VariantsContainerReturnMut::Container(ret_container) =
                            ret.get_mut(&current_idx)
                        {
                            if let VariantsContainer::Container(ret_inner) = ret_container {
                                for i in 0..inner.len() {
                                    match inner[i] {
                                        VariantsContainer::Container(_) => {
                                            ret_inner
                                                .push(VariantsContainer::Container(Vec::new()));
                                        }
                                        VariantsContainer::Variants(_) => {
                                            ret_inner.push(VariantsContainer::Variants(Vec::new()));
                                        }
                                    }
                                    stack.push([current_idx.as_slice(), &[i]].concat());
                                }
                            }
                        }
                    }
                    VariantsContainer::Variants(inner) => {
                        if let VariantsContainerReturnMut::Container(ret_container) =
                            ret.get_mut(&current_idx)
                        {
                            if let VariantsContainer::Variants(ret_inner) = ret_container {
                                for variant in inner {
                                    ret_inner.push(cb(variant.to_owned()));
                                }
                            }
                        }
                    }
                }
            }
        }
        return ret;
    }
}
