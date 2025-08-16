use bevy_declarative_ui_macro::ui_layout;

fn main() {
    println!("Hello, world!");
}

#[ui_layout("/assets/layout.xml")]
pub struct ExampleUIPlugin;
