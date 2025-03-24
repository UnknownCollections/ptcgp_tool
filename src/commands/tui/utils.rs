#![allow(dead_code)]

use cursive::traits::*;
use cursive::view::SizeConstraint;
use cursive::views::{Button, Checkbox, EditView, LinearLayout, ResizedView, TextView};
use cursive::Cursive;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::Arc;

/// Represents the type of browsing (file, folder, save-file).
pub enum BrowseType {
    File,
    Folder,
    SaveFile,
}

/// Creates a horizontal layout with:
///  - A label (e.g., "APK file: ")
///  - A text view that holds the chosen path
///  - A "Browse" button that opens the appropriate file/folder dialog.
pub fn make_path_input(
    label: &str,
    name: &str,
    browse_type: BrowseType,
    filter: Option<(&'static str, &'static [&'static str])>,
) -> LinearLayout {
    let name_rc = Arc::new(name.to_string());
    let label_rc = Arc::new(label.to_string());
    let filter_rc = Arc::new(filter);

    let name_ = Arc::clone(&name_rc);
    let label_ = Arc::clone(&label_rc);
    let filter_ = Arc::clone(&filter_rc);
    let button_cb = move |s: &mut Cursive| {
        let dialog = FileDialog::new().set_title(&*label_);
        let dialog = if let Some((file_type, filter)) = *filter_ {
            dialog.add_filter(file_type, filter)
        } else {
            dialog
        };
        let maybe_path = match browse_type {
            BrowseType::File => dialog.pick_file(),
            BrowseType::Folder => dialog.pick_folder(),
            BrowseType::SaveFile => dialog.save_file(),
        };

        if let Some(path) = maybe_path {
            let path_str = path.display().to_string();
            // Use the Arc's contents
            s.call_on_name(&name_, |view: &mut EditView| {
                view.set_content(path_str);
            });
        }
    };

    LinearLayout::horizontal()
        .child(TextView::new(label))
        .child(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Fixed(1),
            EditView::new().disabled().with_name(name),
        ))
        .child(Button::new_raw(" <Browse>", button_cb))
}

/// Retrieves the content of the named `TextView` as a required `PathBuf`.
/// Panics if the view isn't found (the `.unwrap()`).
pub fn get_required_path(s: &mut Cursive, name: &str) -> PathBuf {
    s.call_on_name(name, |view: &mut EditView| {
        PathBuf::from(&*view.get_content())
    })
    .unwrap()
}

/// Retrieves the content of the named `TextView` as an optional `PathBuf`.
/// Returns `None` if the string is empty.
pub fn get_optional_path(s: &mut Cursive, name: &str) -> Option<PathBuf> {
    s.call_on_name(name, |view: &mut EditView| {
        let value = view.get_content();
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(&*value))
        }
    })
    .unwrap()
}

/// Retrieves the checked state of the named `Checkbox`.
pub fn get_checkbox_value(s: &mut Cursive, name: &str) -> bool {
    s.call_on_name(name, |view: &mut Checkbox| view.is_checked())
        .unwrap()
}
