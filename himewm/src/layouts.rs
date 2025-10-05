use crate::{directories::*, util};
use himewm_layout::{layout::*, user_layout::*};

pub fn initialize_layouts(
    warnings_string: &mut String,
) -> Option<Vec<(std::path::PathBuf, Layout)>> {
    let mut ret = Vec::new();
    let dirs = Directories::new();
    for entry_result in std::fs::read_dir(dirs.layouts_dir).unwrap() {
        match entry_result {
            Ok(entry) => match std::fs::read(entry.path()) {
                Ok(byte_vector) => {
                    let layout_name = std::path::Path::new(&entry.file_name()).with_extension("");
                    let user_layout: UserLayout =
                        match serde_json::from_slice(byte_vector.as_slice()) {
                            Ok(val) => val,
                            Err(e) => {
                                util::add_to_message(
                                    warnings_string,
                                    &format!(
                                        "Warning: An error occurred when parsing layout {}:\n{}",
                                        layout_name.display(),
                                        e
                                    ),
                                );
                                continue;
                            }
                        };
                    let layout = match Layout::try_from(user_layout) {
                        Ok(l) => l,
                        Err(e) => {
                            util::add_to_message(
                                warnings_string,
                                &format!(
                                    "Warning: An error occurred when parsing layout {}:\n{}",
                                    layout_name.display(),
                                    e
                                ),
                            );
                            continue;
                        }
                    };
                    ret.push((layout_name, layout));
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }
    if ret.is_empty() {
        return None;
    } else {
        return Some(ret);
    }
}

pub fn get_layout_idx_map(
    layout_vector: &Vec<(std::path::PathBuf, Layout)>,
) -> std::collections::HashMap<String, usize> {
    let mut ret = std::collections::HashMap::new();
    for (i, (layout_name, _)) in layout_vector.iter().enumerate() {
        ret.insert(layout_name.to_str().unwrap().to_owned(), i);
    }
    return ret;
}
