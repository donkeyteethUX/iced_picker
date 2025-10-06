use iced::widget::{Column, Space, button, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Font, Length, Padding, Theme};
use std::collections::HashMap;
use std::hash::Hash;

/// A tree-based picker item for nested hierarchical data
#[derive(Debug, Clone)]
pub struct PickerItem<Message: Clone, Id: Clone + Eq + Hash> {
    pub id: Id,
    pub label: String,
    pub icon: Option<char>,
    pub icon_font: Option<Font>,
    pub children: Vec<PickerItem<Message, Id>>,
    pub selectable: bool,
    _phantom: std::marker::PhantomData<Message>,
}

impl<Message: Clone, Id: Clone + Eq + Hash> PickerItem<Message, Id> {
    pub fn new(id: Id, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            icon_font: None,
            children: Vec::new(),
            selectable: true,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<char>, font: Font) -> Self {
        let icon = icon.into();
        self.icon = Some(icon);
        self.icon_font = Some(font);
        self
    }

    pub fn with_children(mut self, children: Vec<PickerItem<Message, Id>>) -> Self {
        self.children = children;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
}

/// Creates a picker view from a list of items
pub fn picker<'a, Message: Clone + 'a, Id: Clone + Eq + Hash + 'a>(
    items: &'a [PickerItem<Message, Id>],
    expanded_items: &'a HashMap<Id, bool>,
    on_toggle: &'a dyn Fn(Id) -> Message,
    on_select: Option<&'a dyn Fn(Id) -> Message>,
    indent_size: f32,
) -> Element<'a, Message> {
    let mut content = Column::new().spacing(2);

    for item in items {
        content = content.push(render_item(
            item,
            expanded_items,
            on_toggle,
            on_select,
            indent_size,
            0,
        ));
    }

    scrollable(content).into()
}

fn render_item<'a, Message: Clone + 'a, Id: Clone + Eq + Hash + 'a>(
    item: &'a PickerItem<Message, Id>,
    expanded_items: &'a HashMap<Id, bool>,
    on_toggle: &'a dyn Fn(Id) -> Message,
    on_select: Option<&'a dyn Fn(Id) -> Message>,
    indent_size: f32,
    depth: usize,
) -> Element<'a, Message> {
    let has_children = !item.children.is_empty();
    let is_expanded = expanded_items.get(&item.id).copied().unwrap_or(false);
    let arrow = if is_expanded { "▼" } else { "▶" };
    let indent = Space::with_width(Length::Fixed(depth as f32 * indent_size));

    // Toggle button (arrow)
    let toggle_button = if has_children {
        button(text(arrow).size(12))
            .padding(4)
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    button::Status::Active => button::Style {
                        background: None,
                        text_color: palette.background.base.text,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        snap: Default::default(),
                    },
                    button::Status::Hovered => button::Style {
                        background: Some(iced::Background::Color(palette.background.weak.color)),
                        text_color: palette.background.base.text,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        snap: Default::default(),
                    },
                    button::Status::Pressed => button::Style {
                        background: Some(iced::Background::Color(palette.background.strong.color)),
                        text_color: palette.background.base.text,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        snap: Default::default(),
                    },
                    button::Status::Disabled => button::Style {
                        background: None,
                        text_color: Color::from_rgb(0.5, 0.5, 0.5),
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        snap: Default::default(),
                    },
                }
            })
            .on_press(on_toggle(item.id.clone()))
    } else {
        button(text(arrow).size(12))
            .padding(4)
            .style(|_theme: &Theme, _status| button::Style {
                background: None,
                text_color: Color::TRANSPARENT,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: Default::default(),
            })
    };

    // Create the main row with toggle, icon, and label
    let mut item_row = row![indent, toggle_button]
        .spacing(4)
        .align_y(Alignment::Start);

    // Icon if present
    if let Some(icon_char) = item.icon {
        let mut text = text(format!("{icon_char}")).size(14);
        if let Some(font) = item.icon_font {
            text = text.font(font);
        }
        item_row = item_row.push(text);
    };

    // Label
    item_row = item_row.push(text(&item.label).size(14));

    // Wrap in button if selectable
    let padding = Padding::ZERO; // No padding between rows for compactness.
    let non_button_row = |row| container(row).padding(padding).width(Length::Fill).into();

    let item_element: Element<'a, Message> = if item.selectable {
        if let Some(select_fn) = on_select {
            button(item_row)
                .padding(padding)
                .width(Length::Fill)
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => button::Style {
                            background: None,
                            text_color: palette.background.base.text,
                            ..Default::default()
                        },
                        button::Status::Hovered => button::Style {
                            background: Some(iced::Background::Color(palette.primary.weak.color)),
                            text_color: palette.background.base.text,
                            ..Default::default()
                        },
                        button::Status::Pressed => button::Style {
                            background: Some(iced::Background::Color(palette.primary.strong.color)),
                            text_color: palette.primary.strong.text,
                            ..Default::default()
                        },
                        button::Status::Disabled => button::Style {
                            background: None,
                            text_color: Color::from_rgb(0.5, 0.5, 0.5),
                            ..Default::default()
                        },
                    }
                })
                .on_press(select_fn(item.id.clone()))
                .into()
        } else {
            non_button_row(item_row)
        }
    } else {
        non_button_row(item_row)
    };

    // Build the column with the item and its children (if expanded)
    let mut col = Column::new().push(item_element);

    if has_children && is_expanded {
        for child in &item.children {
            col = col.push(render_item(
                child,
                expanded_items,
                on_toggle,
                on_select,
                indent_size,
                depth + 1,
            ));
        }
    }

    col.into()
}
