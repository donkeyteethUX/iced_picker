use iced::widget::{column, container, text};
use iced::{Element, Length, Task, Theme};
use iced_fonts::{Nerd, NERD_FONT, NERD_FONT_BYTES};
use iced_picker::{picker, PickerItem};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn main() -> iced::Result {
    iced::application("File Picker Example", update, view)
        .theme(|_| Theme::Dracula)
        .font(NERD_FONT_BYTES)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    ToggleFolder(PathBuf),
    SelectItem(PathBuf),
}

struct State {
    root_path: PathBuf,
    items: Vec<PickerItem<Message, PathBuf>>,
    expanded_folders: HashMap<PathBuf, bool>,
    selected_path: Option<PathBuf>,
}

impl Default for State {
    fn default() -> Self {
        let root_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut expanded_folders = HashMap::new();
        // Expand the root by default
        expanded_folders.insert(root_path.clone(), true);
        let items = build_tree(&root_path);

        State {
            root_path,
            items,
            expanded_folders,
            selected_path: None,
        }
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ToggleFolder(path) => {
            let is_expanded = state.expanded_folders.get(&path).copied().unwrap_or(false);
            state.expanded_folders.insert(path, !is_expanded);
        }
        Message::SelectItem(path) => {
            state.selected_path = Some(path);
        }
    }
    Task::none()
}

fn view(state: &State) -> Element<Message> {
    let picker_view = container(picker(
        &state.items,
        &state.expanded_folders,
        &Message::ToggleFolder,
        Some(&Message::SelectItem),
        10.0,
    ))
    .padding(5)
    .height(Length::Fill);

    let selected_text = if let Some(path) = &state.selected_path {
        format!("Selected: {}", path.display())
    } else {
        String::from("No item selected")
    };

    let content = column![
        text("File Picker Example").size(24),
        text(format!("Root: {}", state.root_path.display())).size(12),
        text(selected_text).size(14),
        container(picker_view)
            .height(Length::Fill)
            .width(Length::Fill),
    ]
    .spacing(10)
    .padding(20);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn build_tree(path: &Path) -> Vec<PickerItem<Message, PathBuf>> {
    let mut items = Vec::new();

    // Try to read the directory
    if let Ok(entries) = fs::read_dir(path) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();

        // Sort: directories first, then files, both alphabetically
        entries.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for entry in entries {
            let entry_path = entry.path();

            // Skip hidden files (starting with .)
            if let Some(file_name) = entry_path.file_name() {
                if file_name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }

            if entry_path.is_dir() {
                // Always build the full tree structure - the picker widget will handle showing/hiding
                let children = build_tree(&entry_path);
                let dir_name = entry.file_name().to_string_lossy().to_string();

                items.push(
                    PickerItem::new(entry_path.clone(), dir_name)
                        .with_icon(Nerd::Folder, NERD_FONT)
                        .with_children(children)
                        .selectable(true),
                );
            } else {
                let file_name = entry.file_name().to_string_lossy().to_string();
                items.push(
                    PickerItem::new(entry_path.clone(), file_name)
                        .with_icon(Nerd::File, NERD_FONT)
                        .selectable(true),
                );
            }
        }
    }

    items
}
