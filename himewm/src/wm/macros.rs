macro_rules! window_info {
    ($wm:expr, $hwnd:expr $(,$guard:expr)? $(;$else:expr)?) => {
        match $wm.window_info.get(&$hwnd.0) {
            Some(val) $(if val.restored == $guard)? => val,
            _ => {
                $($else)?
                return;
            }
        }
    };
}

macro_rules! window_info_mut {
    ($wm:expr, $hwnd:expr $(,$guard:expr)? $(;$else:expr)?) => {
        match $wm.window_info.get_mut(&$hwnd.0) {
            Some(val) $(if val.restored == $guard)? => val,
            _ => {
                $($else)?
                return;
            }
        }
    };
}

macro_rules! window_info_owned {
    ($wm:expr, $hwnd:expr $(,$guard:expr)? $(;$else:expr)?) => {
        match $wm.window_info.get(&$hwnd.0) {
            Some(val) $(if val.restored == $guard)? => val.to_owned(),
            _ => {
                $($else)?
                return;
            }
        }
    };
}
