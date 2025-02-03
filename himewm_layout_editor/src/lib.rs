use enums::{Color, FrameType};
use fltk::{group::FlexType, *};
use fltk_theme::*;
use group::{PackType, ScrollType};
use himewm_layout::*;
use prelude::{GroupExt, WidgetBase, WidgetExt};

#[derive(Clone)]
enum Message {
    SelectedVariantChanged(usize),
    SelectedLayoutStateChanged(usize),
    SelectedZoneChanged(usize),
    NewLayoutState,
    CloneLayoutState(usize),
    SwapLayoutStates(usize, usize),
    DeleteLayoutState(usize),

}

struct LayoutEditor {
    layout_group: LayoutGroup,
    selected_variant_idx: usize,
    selected_layout_state_idx: usize,
    selected_zone_idx1: Option<usize>,
    selected_zone_idx2: Option<usize>,
}

impl LayoutEditor {
    fn new(layout: LayoutGroup) -> Self {
        let default_idx = layout.default_idx();

        return LayoutEditor {
            layout_group: layout,
            selected_variant_idx: default_idx,
            selected_layout_state_idx: 0,
            selected_zone_idx1: None,
            selected_zone_idx2: None,
        };
    }
}

struct EditorWidgets {
    editor: LayoutEditor,
    variant_list: group::Scroll,
    layout_state_selection: group::Scroll,
    layout_state_buttons: group::Flex,
    layout_state_pack: group::Pack,
    layout_state_display: group::Group,
}

impl EditorWidgets {
    fn initialize(layout: LayoutGroup, sender: &app::Sender<Message>) -> Self {
        let variant_list = Self::create_variant_list(&layout, sender);

        let layout_state_selection = Self::create_layout_state_selection(&layout, sender);

        let layout_state_buttons = Self::create_layout_state_buttons(sender).below_of(&layout_state_selection, 4);

        let layout_state_pack_h= layout_state_selection.h() + layout_state_buttons.h();

        let mut layout_state_pack = group::Pack::default().with_size(layout_state_selection.w(), layout_state_pack_h);

        layout_state_pack.end();
        
        layout_state_pack.set_spacing(4);

        layout_state_pack.resize_callback(move |p, _, _, _, h| {
            if h != layout_state_pack_h {
                p.widget_resize(p.x(), p.y(), p.w(), layout_state_pack_h); 
            }
        });

        layout_state_pack.add(&layout_state_selection);

        layout_state_pack.add(&layout_state_buttons);

        let layout_state_display = Self::create_layout_state_display(&layout, sender);

        let editor = LayoutEditor::new(layout);

        let mut ret = EditorWidgets {
            editor,
            variant_list,
            layout_state_selection,
            layout_state_buttons,
            layout_state_pack,
            layout_state_display,
        };

        ret.update_highlighted_variant(
            ret.editor.selected_variant_idx,
            ret.editor.selected_variant_idx,
        );

        ret.update_shown_layout_state_selection(
            ret.editor.selected_variant_idx,
            ret.editor.selected_variant_idx,
        );

        ret.update_highlighted_layout_state_button(
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_layout_state_idx,
            ),
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_layout_state_idx,
            ),
        );

        ret.update_shown_layout_state(
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_layout_state_idx,
            ),
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_layout_state_idx,
            ),
        );

        return ret;
    }

    fn create_variant_list(layout: &LayoutGroup, sender: &app::Sender<Message>) -> group::Scroll {
        let mut scroll = group::Scroll::default_fill().with_type(ScrollType::Vertical);

        scroll.set_size(scroll.w() / 8, scroll.h() / 2);

        scroll.set_color(Color::Background2);

        scroll.resize_callback(|s, _, _, w, _| {
            if let Some(p) = &mut s.child(0) {
                p.set_size(w, p.h());
            }
        });

        let pack = group::Pack::default_fill().with_type(PackType::Vertical);

        for i in 0..layout.layouts_len() {
            let mut b = button::Button::default().with_size(0, 20);

            b.set_label_size(16);

            b.set_color(colors::html::DodgerBlue);

            b.set_frame(FrameType::NoBox);

            if i == layout.default_idx() {
                b.set_label(format!("{i} (default)").as_str());
            } else {
                b.set_label(i.to_string().as_str());
            }

            b.emit(sender.clone(), Message::SelectedVariantChanged(i));
        }

        pack.end();

        scroll.end();

        return scroll;
    }

    fn create_layout_state_selection(
        layout: &LayoutGroup,
        sender: &app::Sender<Message>,
    ) -> group::Scroll {
        let mut scroll = group::Scroll::default_fill().with_type(ScrollType::Horizontal);

        scroll.set_size(scroll.w() / 2, 72);

        // Any styling of the scrollbar should probably happen here

        scroll.set_color(Color::Background2);

        for variant in layout.get_layouts() {
            let mut pack = group::Pack::default()
                .with_size(scroll.w() - 8, 64)
                .with_type(PackType::Horizontal)
                .center_of_parent();

            pack.set_spacing(4);

            for j in 0..variant.manual_zones_until() {
                let mut b = button::Button::default()
                    .with_size(64, 0)
                    .with_label((j + 1).to_string().as_str());

                b.set_color(Color::Background2);

                b.set_selection_color(Color::Background);

                b.emit(sender.clone(), Message::SelectedLayoutStateChanged(j));
            }

            pack.end();

            pack.hide();
        }

        scroll.end();

        return scroll;
    }

    fn create_layout_state_buttons(sender: &app::Sender<Message>) -> group::Flex {
        let mut flex = group::Flex::default_fill().with_type(FlexType::Row);

        let new_width = flex.w()/2;

        WidgetExt::set_size(&mut flex, new_width, 32);

        flex.set_pad(4);

        let button_new = button::Button::default().with_label("New");

        let button_duplicate = button::Button::default().with_label("Duplicate");

        let button_delete = button::Button::default().with_label("Delete");

        let _frame = frame::Frame::default();
        
        let button_left = button::Button::default().with_label("@<");
        
        let button_right = button::Button::default().with_label("@>");
        
        flex.fixed(&button_new, 64);
        
        flex.fixed(&button_duplicate, 80);
        
        flex.fixed(&button_delete, 64);
        
        flex.fixed(&button_left, 32);
        
        flex.fixed(&button_right, 32);

        flex.end();

        return flex;

    }

    fn create_layout_state_display(layout: &LayoutGroup, sender: &app::Sender<Message>) -> group::Group {
        let mut group = group::Group::default_fill();

        for variant in layout.get_layouts() {
            let mut variant_group = group::Group::default_fill();

            for i in 0..variant.manual_zones_until() {
                let mut g = Self::group_widget_from_layout_at(variant, i, sender);

                g.hide();
            }

            variant_group.widget_resize(
                variant_group.x(),
                variant_group.y(),
                variant_group.w() / 2,
                variant_group.h() / 2,
            );

            variant_group.end();

            variant_group.hide();
        }

        group.widget_resize(group.x(), group.y(), group.w() / 2, group.h() / 2);

        group.end();

        return group;
    }

    fn group_widget_from_layout_at(
        layout: &Layout,
        idx: usize,
        sender: &app::Sender<Message>,
    ) -> group::Group {
        let mut group = group::Group::default_fill();

        let w = group.w() / 2;

        let h = group.h() / 2;

        group.set_size(w, h);

        let layout_state = &layout.get_zones()[idx];

        let layout_width = layout.get_monitor_rect().right as f64;

        let layout_height = layout.get_monitor_rect().bottom as f64;

        for (i, zone) in layout_state.iter().enumerate() {
            let mut b = button::Button::new(
                ((zone.left as f64 * w as f64) / layout_width).round() as i32,
                ((zone.top as f64 * h as f64) / layout_height).round() as i32,
                ((zone.w() as f64 * w as f64) / layout_width).round() as i32,
                ((zone.h() as f64 * h as f64) / layout_height).round() as i32,
                Some((i + 1).to_string().as_str()),
            );

            // TODO: this frame type doesn't look too great - probably
            // figure out how to make it look better
            b.set_frame(FrameType::EmbossedBox);

            b.set_label_size(36);

            b.set_label_color(Color::Black);

            b.set_color(colors::html::Gainsboro);

            b.set_selection_color(colors::html::DodgerBlue);

            b.emit(sender.clone(), Message::SelectedZoneChanged(i));
        }

        group.end();

        return group;
    }

    fn update_highlighted_variant(&mut self, old_idx: usize, new_idx: usize) {
        if let Some(p) = &self.variant_list.child(0) {
            let pack = group::Pack::from_dyn_widget(p).unwrap();

            if let Some(old_button) = &mut pack.child(old_idx as i32) {
                old_button.set_frame(FrameType::NoBox);
            }

            if let Some(new_button) = &mut pack.child(new_idx as i32) {
                new_button.set_frame(FrameType::UpBox);
            }
        }
    }

    fn update_shown_layout_state_selection(&mut self, old_idx: usize, new_idx: usize) {
        if let Some(old_pack) = &mut self.layout_state_selection.child(old_idx as i32) {
            old_pack.hide();
        }

        if let Some(new_pack) = &mut self.layout_state_selection.child(new_idx as i32) {
            new_pack.show();
        }
    }

    fn update_highlighted_layout_state_button(
        &mut self,
        old_idx: (usize, usize),
        new_idx: (usize, usize),
    ) {
        if let Some(old_pack) = &mut self.layout_state_selection.child(old_idx.0 as i32) {
            if let Some(old_button) = &mut group::Pack::from_dyn_widget(old_pack)
                .unwrap()
                .child(old_idx.1 as i32)
            {
                old_button.set_color(Color::Background2);
            }

            old_pack.hide();
        }

        if let Some(new_pack) = &mut self.layout_state_selection.child(new_idx.0 as i32) {
            if let Some(new_button) = &mut group::Pack::from_dyn_widget(new_pack)
                .unwrap()
                .child(new_idx.1 as i32)
            {
                new_button.set_color(colors::html::DimGray);
            }

            new_pack.show();
        }
    }

    fn update_shown_layout_state(&mut self, old_idx: (usize, usize), new_idx: (usize, usize)) {
        if let Some(old_variant_group) = &mut self.layout_state_display.child(old_idx.0 as i32) {
            if let Some(old_group) = &mut group::Group::from_dyn_widget(old_variant_group)
                .unwrap()
                .child(old_idx.1 as i32)
            {
                old_group.hide();
            }

            old_variant_group.hide();
        }

        if let Some(new_variant_group) = &mut self.layout_state_display.child(new_idx.0 as i32) {
            if let Some(new_group) = &mut group::Group::from_dyn_widget(new_variant_group)
                .unwrap()
                .child(new_idx.1 as i32)
            {
                new_group.show();
            }

            new_variant_group.show();
        }
    }
}

pub struct LayoutEditorGUI {
    app: app::App,
    window: window::Window,
    sender: app::Sender<Message>,
    receiver: app::Receiver<Message>,
    editor_widgets: Option<EditorWidgets>,
}

impl LayoutEditorGUI {
    pub fn create() -> Self {
        initialize_colours();

        let app = app::App::default();

        let widget_scheme = WidgetScheme::new(SchemeType::Fluent);

        widget_scheme.apply();

        let (sender, receiver) = app::channel();

        return LayoutEditorGUI {
            app,
            window: create_window(),
            sender,
            receiver,
            editor_widgets: None,
        };
    }

    pub fn edit_layout(&mut self, layout: LayoutGroup) {
        self.window.begin();

        self.editor_widgets = Some(EditorWidgets::initialize(layout, &self.sender));

        self.window.end();

        // Test code

        if let Some(editor) = &mut self.editor_widgets {

            editor.layout_state_pack.set_pos(self.window.w()/2 - editor.layout_state_pack.w()/2, 0);

            editor.layout_state_display.set_pos(
                self.window.w() / 2 - editor.layout_state_display.w() / 2,
                self.window.h() / 2 - editor.layout_state_display.h() / 2,
            );
        }
    }

    fn handle_events(&mut self) {
        let editor_widgets = match &mut self.editor_widgets {
            Some(val) => val,

            None => return,
        };

        if let Some(msg) = self.receiver.recv() {
            match msg {
                Message::SelectedVariantChanged(idx) => {
                    let old_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let old_layout_state_idx = editor_widgets.editor.selected_layout_state_idx;

                    editor_widgets.editor.selected_variant_idx = idx;

                    editor_widgets.editor.selected_layout_state_idx = 0;

                    editor_widgets.update_highlighted_variant(old_variant_idx, idx);

                    editor_widgets.update_shown_layout_state_selection(old_variant_idx, idx);

                    editor_widgets.update_highlighted_layout_state_button(
                        (old_variant_idx, old_layout_state_idx),
                        (idx, 0),
                    );

                    editor_widgets.update_shown_layout_state((old_variant_idx, old_layout_state_idx), (idx, 0));
                }

                Message::SelectedLayoutStateChanged(idx) => {
                    let variant_idx = editor_widgets.editor.selected_variant_idx;

                    let old_idx = editor_widgets.editor.selected_layout_state_idx;

                    editor_widgets.editor.selected_layout_state_idx = idx;

                    editor_widgets.update_highlighted_layout_state_button(
                        (variant_idx, old_idx),
                        (variant_idx, idx),
                    );

                    editor_widgets.update_shown_layout_state((variant_idx, old_idx), (variant_idx, idx));
                }

                Message::SelectedZoneChanged(idx) => {}

                _ => ()
            }
        }
    }

    pub fn run(mut self) {
        self.window.show();

        while self.app.wait() {
            self.handle_events();
        }
    }
}

fn initialize_colours() {
    app::background(16, 16, 16);

    app::background2(32, 32, 32);
}

fn create_window() -> window::Window {
    let primary_screen = app::Screen::new(0).unwrap();

    let mut window = window::Window::new(
        primary_screen.w() / 4,
        primary_screen.h() / 4,
        primary_screen.w() / 2,
        primary_screen.h() / 2,
        "PLACEHOLDER",
    );

    window.make_resizable(true);

    window.end();

    return window;
}
