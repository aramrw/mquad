use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets::InputText};

fn main() {
    let mut text = String::new();
    InputText::new(hash!()).ui(&mut root_ui(), &mut text);
}
