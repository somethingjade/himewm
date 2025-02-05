use enums::{Color, FrameType};
use fltk::{group::FlexType, *};
use fltk_theme::*;
use group::{PackType, ScrollType};
use himewm_layout::*;
use prelude::{GroupExt, WidgetBase, WidgetExt};

#[derive(Clone)]
enum SwapDirection {
    Previous,
    Next,
}

#[derive(Clone)]
enum Message {
    SelectedVariantChanged(usize),
    SelectedVariantStateChanged(usize),
    SelectedZoneChanged(usize),
    NewVariantState,
    DuplicateVariantState,
    DeleteVariantState,
    SwapVariantState(SwapDirection),
}

struct LayoutEditor {
    layout: Layout,
    selected_variant_idx: usize,
    selected_variant_state_idx: usize,
    selected_zone_idx1: Option<usize>,
    selected_zone_idx2: Option<usize>,
}

impl LayoutEditor {
    fn new(layout: Layout) -> Self {
        let default_idx = layout.default_idx();

        return LayoutEditor {
            layout,
            selected_variant_idx: default_idx,
            selected_variant_state_idx: 0,
            selected_zone_idx1: None,
            selected_zone_idx2: None,
        };
    }
}

struct EditorWidgets {
    editor: LayoutEditor,
    variant_list: group::Scroll,
    variant_state_selection: group::Scroll,
    variant_state_pack: group::Pack,
    variant_state_display: group::Group,
}

impl EditorWidgets {
    fn initialize(layout: Layout, sender: &app::Sender<Message>) -> Self {
        let variant_list = Self::create_variant_list(&layout, sender);

        let variant_state_selection = Self::create_variant_state_selection(&layout, sender);

        let variant_state_buttons =
            Self::create_variant_state_buttons(sender).below_of(&variant_state_selection, 4);

        let variant_state_pack_h = variant_state_selection.h() + variant_state_buttons.h();

        let mut variant_state_pack =
            group::Pack::default().with_size(variant_state_selection.w(), variant_state_pack_h);

        variant_state_pack.end();

        variant_state_pack.set_spacing(4);

        variant_state_pack.resize_callback(move |p, _, _, _, h| {
            if h != variant_state_pack_h {
                p.widget_resize(p.x(), p.y(), p.w(), variant_state_pack_h);
            }
        });

        variant_state_pack.add(&variant_state_selection);

        variant_state_pack.add(&variant_state_buttons);

        let variant_state_display = Self::create_variant_state_display(&layout, sender);

        let editor = LayoutEditor::new(layout);

        let mut ret = EditorWidgets {
            editor,
            variant_list,
            variant_state_selection,
            variant_state_pack,
            variant_state_display,
        };

        ret.update_highlighted_variant(
            ret.editor.selected_variant_idx,
            ret.editor.selected_variant_idx,
        );

        ret.update_shown_variant_state_selection(
            ret.editor.selected_variant_idx,
            ret.editor.selected_variant_idx,
        );

        ret.update_highlighted_variant_state_button(
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_variant_state_idx,
            ),
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_variant_state_idx,
            ),
        );

        ret.update_shown_variant_state(
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_variant_state_idx,
            ),
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_variant_state_idx,
            ),
        );

        return ret;
    }

    fn display_group_from_variant_state(
        variant: &Variant,
        idx: usize,
        sender: &app::Sender<Message>,
    ) -> group::Group {
        let group = group::Group::default_fill();

        let w = group.w();

        let h = group.h();

        let x_offset = group.x();

        let y_offset = group.y();

        let zones = &variant.get_zones()[idx];

        let variant_width = variant.get_monitor_rect().right as f64;

        let variant_height = variant.get_monitor_rect().bottom as f64;

        for (i, zone) in zones.iter().enumerate() {
            let mut b = button::Button::new(
                ((zone.left as f64 * w as f64) / variant_width).round() as i32 + x_offset,
                ((zone.top as f64 * h as f64) / variant_height).round() as i32 + y_offset,
                ((zone.w() as f64 * w as f64) / variant_width).round() as i32,
                ((zone.h() as f64 * h as f64) / variant_height).round() as i32,
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

    fn create_variant_list(variant: &Layout, sender: &app::Sender<Message>) -> group::Scroll {
        let mut scroll = group::Scroll::default_fill().with_type(ScrollType::Vertical);

        scroll.set_size(scroll.w() / 8, scroll.h() / 2);

        scroll.set_color(Color::Background2);

        scroll.resize_callback(|s, _, _, w, _| {
            if let Some(p) = &mut s.child(0) {
                p.set_size(w, p.h());
            }
        });

        let pack = group::Pack::default_fill().with_type(PackType::Vertical);

        for i in 0..variant.variants_len() {
            let mut b = button::Button::default().with_size(0, 20);

            b.set_label_size(16);

            b.set_color(colors::html::DodgerBlue);

            b.set_frame(FrameType::NoBox);

            if i == variant.default_idx() {
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

    fn create_variant_state_selection(
        variant: &Layout,
        sender: &app::Sender<Message>,
    ) -> group::Scroll {
        let mut scroll = group::Scroll::default_fill().with_type(ScrollType::Horizontal);

        scroll.set_size(scroll.w() / 2, 72);

        // Any styling of the scrollbar should probably happen here

        scroll.set_color(Color::Background2);

        for variant in variant.get_variants() {
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

                b.emit(sender.clone(), Message::SelectedVariantStateChanged(j));
            }

            pack.end();

            pack.hide();
        }

        scroll.end();

        return scroll;
    }

    fn create_variant_state_buttons(sender: &app::Sender<Message>) -> group::Flex {
        let mut flex = group::Flex::default_fill().with_type(FlexType::Row);

        let new_width = flex.w() / 2;

        WidgetExt::set_size(&mut flex, new_width, 32);

        flex.set_pad(4);

        let mut button_new = button::Button::default().with_label("New");

        let mut button_duplicate = button::Button::default().with_label("Duplicate");

        let mut button_delete = button::Button::default().with_label("Delete");

        let _frame = frame::Frame::default();

        let mut button_left = button::Button::default().with_label("@<");

        let mut button_right = button::Button::default().with_label("@>");

        flex.fixed(&button_new, 64);

        flex.fixed(&button_duplicate, 80);

        flex.fixed(&button_delete, 64);

        flex.fixed(&button_left, 32);

        flex.fixed(&button_right, 32);

        flex.end();

        button_new.emit(sender.clone(), Message::NewVariantState);

        button_duplicate.emit(sender.clone(), Message::DuplicateVariantState);

        button_delete.emit(sender.clone(), Message::DeleteVariantState);

        button_left.emit(
            sender.clone(),
            Message::SwapVariantState(SwapDirection::Previous),
        );

        button_right.emit(
            sender.clone(),
            Message::SwapVariantState(SwapDirection::Next),
        );

        return flex;
    }

    fn create_variant_state_display(
        layout: &Layout,
        sender: &app::Sender<Message>,
    ) -> group::Group {
        let mut group = group::Group::default_fill();

        group.set_size(group.w() / 2, group.h() / 2);

        for variant in layout.get_variants() {
            let mut layout = group::Group::default_fill();

            for i in 0..variant.manual_zones_until() {
                let mut g = Self::display_group_from_variant_state(variant, i, sender);

                g.hide();
            }

            layout.end();

            layout.hide();
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

    fn update_shown_variant_state_selection(&mut self, old_idx: usize, new_idx: usize) {
        if let Some(old_pack) = &mut self.variant_state_selection.child(old_idx as i32) {
            old_pack.hide();
        }

        if let Some(new_pack) = &mut self.variant_state_selection.child(new_idx as i32) {
            new_pack.show();
        }
    }

    fn update_highlighted_variant_state_button(
        &mut self,
        old_idx: (usize, usize),
        new_idx: (usize, usize),
    ) {
        if let Some(old_pack) = &mut self.variant_state_selection.child(old_idx.0 as i32) {
            if let Some(old_button) = &mut group::Pack::from_dyn_widget(old_pack)
                .unwrap()
                .child(old_idx.1 as i32)
            {
                old_button.set_color(Color::Background2);
            }

            old_pack.hide();
        }

        if let Some(new_pack) = &mut self.variant_state_selection.child(new_idx.0 as i32) {
            if let Some(new_button) = &mut group::Pack::from_dyn_widget(new_pack)
                .unwrap()
                .child(new_idx.1 as i32)
            {
                new_button.set_color(colors::html::DimGray);
            }

            new_pack.show();
        }
    }

    fn update_shown_variant_state(&mut self, old_idx: (usize, usize), new_idx: (usize, usize)) {
        if let Some(old_variant) = &mut self.variant_state_display.child(old_idx.0 as i32) {
            if let Some(old_group) = &mut group::Group::from_dyn_widget(old_variant)
                .unwrap()
                .child(old_idx.1 as i32)
            {
                old_group.hide();
            }

            old_variant.hide();
        }

        if let Some(new_variant) = &mut self.variant_state_display.child(new_idx.0 as i32) {
            if let Some(new_group) = &mut group::Group::from_dyn_widget(new_variant)
                .unwrap()
                .child(new_idx.1 as i32)
            {
                new_group.show();
            }

            new_variant.show();
        }
    }

    fn highlight_selected_zone(&mut self, idx: usize) {
        let selected_variant_idx = self.editor.selected_variant_idx;

        let selected_variant_state_idx = self.editor.selected_variant_state_idx;

        if let Some(variant) = group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(selected_variant_idx as i32)
                .unwrap(),
        ) {
            if let Some(group) = group::Group::from_dyn_widget(
                &variant.child(selected_variant_state_idx as i32).unwrap(),
            ) {
                if let Some(zone_button) =
                    &mut button::Button::from_dyn_widget(&group.child(idx as i32).unwrap())
                {
                    zone_button.set_color(colors::html::DodgerBlue);

                    zone_button.redraw();
                }
            }
        }
    }

    fn dehighlight_zone(&mut self, variant_idx: usize, variant_state_idx: usize, zone_idx: usize) {
        if let Some(variant) = group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        ) {
            if let Some(group) =
                group::Group::from_dyn_widget(&variant.child(variant_state_idx as i32).unwrap())
            {
                if let Some(zone_button) =
                    &mut button::Button::from_dyn_widget(&group.child(zone_idx as i32).unwrap())
                {
                    zone_button.set_color(colors::html::Gainsboro);

                    zone_button.redraw();
                }
            }
        }
    }

    fn new_variant_state(&mut self, sender: &app::Sender<Message>) {
        let variant_idx = self.editor.selected_variant_idx;

        let variant_state_pack = &mut group::Pack::from_dyn_widget(
            &self
                .variant_state_selection
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        variant_state_pack.begin();

        let mut b = button::Button::default()
            .with_size(64, 0)
            .with_label(variant_state_pack.children().to_string().as_str());

        b.set_color(Color::Background2);

        b.set_selection_color(Color::Background);

        b.emit(
            sender.clone(),
            Message::SelectedVariantStateChanged(variant_state_pack.children() as usize - 1),
        );

        variant_state_pack.end();

        let variant_state_display_group = &mut group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        variant_state_display_group.begin();

        let _g = Self::display_group_from_variant_state(
            &self.editor.layout.get_variants()[variant_idx],
            variant_state_pack.children() as usize - 1,
            sender,
        );

        variant_state_display_group.end();

        sender.send(Message::SelectedVariantStateChanged(
            variant_state_pack.children() as usize - 1,
        ));
    }

    fn delete_variant_state(&mut self, sender: &app::Sender<Message>) {
        let variant_idx = self.editor.selected_variant_idx;

        let variant_state_idx = self.editor.selected_variant_state_idx;

        let variant_state_pack = &mut group::Pack::from_dyn_widget(
            &self
                .variant_state_selection
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        WidgetBase::delete(variant_state_pack.child(variant_state_idx as i32).unwrap());

        let variant_state_display_group = &mut group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        WidgetBase::delete(
            variant_state_display_group
                .child(variant_state_idx as i32)
                .unwrap(),
        );

        self.decrement_all_variant_state_buttons_after(variant_idx, variant_state_idx, sender);

        if variant_state_idx
            == self.editor.layout.get_variants()[self.editor.selected_variant_idx]
                .manual_zones_until()
        {
            self.editor.selected_variant_state_idx -= 1;
        }

        sender.send(Message::SelectedVariantStateChanged(
            self.editor.selected_variant_state_idx,
        ));
    }

    fn swap_variant_states(&mut self, swap_with: usize, sender: &app::Sender<Message>) {
        let variant_idx = self.editor.selected_variant_idx;

        let first_idx = std::cmp::min(self.editor.selected_variant_state_idx, swap_with);
        let second_idx = std::cmp::max(self.editor.selected_variant_state_idx, swap_with);

        let variant_state_pack = &mut group::Pack::from_dyn_widget(
            &self
                .variant_state_selection
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        let first_button = &mut button::Button::from_dyn_widget(
            &variant_state_pack.child(first_idx as i32).unwrap(),
        )
        .unwrap();

        let second_button = &mut button::Button::from_dyn_widget(
            &variant_state_pack.child(second_idx as i32).unwrap(),
        )
        .unwrap();

        variant_state_pack.remove_by_index(second_idx as i32);

        variant_state_pack.remove_by_index(first_idx as i32);

        first_button.set_label((second_idx + 1).to_string().as_str());

        first_button.emit(
            sender.clone(),
            Message::SelectedVariantStateChanged(second_idx),
        );

        second_button.set_label((first_idx + 1).to_string().as_str());

        second_button.emit(
            sender.clone(),
            Message::SelectedVariantStateChanged(first_idx),
        );

        variant_state_pack.insert(second_button, first_idx as i32);

        variant_state_pack.insert(first_button, second_idx as i32);

        let variant_state_display_group = &mut group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        let first_variant_state_display = &group::Group::from_dyn_widget(
            &variant_state_display_group.child(first_idx as i32).unwrap(),
        )
        .unwrap();

        let second_variant_state_display = &group::Group::from_dyn_widget(
            &variant_state_display_group
                .child(second_idx as i32)
                .unwrap(),
        )
        .unwrap();

        variant_state_display_group.remove_by_index(second_idx as i32);

        variant_state_display_group.remove_by_index(first_idx as i32);

        variant_state_display_group.insert(second_variant_state_display, first_idx as i32);

        variant_state_display_group.insert(first_variant_state_display, second_idx as i32);

        sender.send(Message::SelectedVariantStateChanged(swap_with));
    }

    fn decrement_all_variant_state_buttons_after(
        &mut self,
        variant_idx: usize,
        variant_state_idx: usize,
        sender: &app::Sender<Message>,
    ) {
        let variant_state_pack = &mut group::Pack::from_dyn_widget(
            &self
                .variant_state_selection
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        for i in variant_state_idx as i32..variant_state_pack.children() {
            let b = &mut button::Button::from_dyn_widget(&variant_state_pack.child(i).unwrap())
                .unwrap();

            b.set_label((i + 1).to_string().as_str());

            b.emit(
                sender.clone(),
                Message::SelectedVariantStateChanged(i as usize),
            );
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

    pub fn edit_layout(&mut self, layout: Layout) {
        self.window.begin();

        self.editor_widgets = Some(EditorWidgets::initialize(layout, &self.sender));

        self.window.end();

        // Test code

        if let Some(editor) = &mut self.editor_widgets {
            editor
                .variant_state_pack
                .set_pos(self.window.w() / 2 - editor.variant_state_pack.w() / 2, 0);

            editor.variant_state_display.set_pos(
                self.window.w() / 2 - editor.variant_state_display.w() / 2,
                self.window.h() / 2 - editor.variant_state_display.h() / 2,
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

                    let old_variant_state_idx = editor_widgets.editor.selected_variant_state_idx;

                    editor_widgets.editor.selected_variant_idx = idx;

                    editor_widgets.editor.selected_variant_state_idx = 0;

                    editor_widgets.update_highlighted_variant(old_variant_idx, idx);

                    editor_widgets.update_shown_variant_state_selection(old_variant_idx, idx);

                    editor_widgets.update_highlighted_variant_state_button(
                        (old_variant_idx, old_variant_state_idx),
                        (idx, 0),
                    );

                    editor_widgets.update_shown_variant_state(
                        (old_variant_idx, old_variant_state_idx),
                        (idx, 0),
                    );
                }

                Message::SelectedVariantStateChanged(idx) => {
                    let variant_idx = editor_widgets.editor.selected_variant_idx;

                    let old_idx = editor_widgets.editor.selected_variant_state_idx;

                    editor_widgets.editor.selected_variant_state_idx = idx;

                    editor_widgets.update_highlighted_variant_state_button(
                        (variant_idx, old_idx),
                        (variant_idx, idx),
                    );

                    editor_widgets
                        .update_shown_variant_state((variant_idx, old_idx), (variant_idx, idx));

                    if let Some(zone_idx) = editor_widgets.editor.selected_zone_idx1 {
                        editor_widgets.dehighlight_zone(variant_idx, old_idx, zone_idx);

                        editor_widgets.editor.selected_zone_idx1 = None;
                    }

                    if let Some(zone_idx) = editor_widgets.editor.selected_zone_idx2 {
                        editor_widgets.dehighlight_zone(variant_idx, old_idx, zone_idx);

                        editor_widgets.editor.selected_zone_idx2 = None;
                    }
                }

                Message::SelectedZoneChanged(idx) => {
                    let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let selected_variant_state_idx =
                        editor_widgets.editor.selected_variant_state_idx;

                    if let Some(old_idx) = editor_widgets.editor.selected_zone_idx1 {
                        editor_widgets.dehighlight_zone(
                            selected_variant_idx,
                            selected_variant_state_idx,
                            old_idx,
                        );

                        if old_idx == idx {
                            return;
                        }
                    }

                    editor_widgets.editor.selected_zone_idx1 = Some(idx);

                    editor_widgets.highlight_selected_zone(idx);
                }

                Message::NewVariantState => {
                    let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx];

                    variant.new_zone_vec();

                    editor_widgets.new_variant_state(&self.sender);
                }

                Message::DuplicateVariantState => {
                    let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx];

                    let idx = editor_widgets.editor.selected_variant_state_idx;

                    variant.clone_zone_vec(idx);

                    editor_widgets.new_variant_state(&self.sender);
                }

                Message::DeleteVariantState => {
                    let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx];

                    let idx = editor_widgets.editor.selected_variant_state_idx;

                    variant.delete_zones(idx);

                    editor_widgets.delete_variant_state(&self.sender);
                }

                Message::SwapVariantState(swap_direction) => {
                    let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx];

                    let selected_variant_state_idx =
                        editor_widgets.editor.selected_variant_state_idx;

                    let swap_with = match swap_direction {
                        SwapDirection::Previous if selected_variant_state_idx != 0 => {
                            selected_variant_state_idx - 1
                        }
                        SwapDirection::Next
                            if editor_widgets.editor.selected_variant_state_idx
                                != variant.manual_zones_until() - 1 =>
                        {
                            selected_variant_state_idx + 1
                        }
                        _ => return,
                    };

                    variant.swap_zone_vectors(selected_variant_state_idx, swap_with);

                    editor_widgets.swap_variant_states(swap_with, &self.sender);
                }
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
