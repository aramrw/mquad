use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets::ComboBox};

fn main() {
    let mut selected = 0;
    ComboBox::new(hash!(), &["Japanese", "Spanish"]).ui(&mut root_ui(), &mut selected);
}
