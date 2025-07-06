echo Run all examples
# Default
cargo r --example menu --no-default-features --features="bevy_ui bevy_text bevy/bevy_winit bevy/bevy_picking"
cargo r --example transform_translation --no-default-features --features="bevy_sprite bevy/bevy_winit bevy/bevy_picking"
cargo r --example transform_rotation --no-default-features --features="bevy_sprite bevy/bevy_winit bevy/bevy_picking"
cargo r --example sequence --no-default-features --features="bevy_sprite bevy_text bevy/bevy_winit bevy/bevy_picking"
# bevy_sprite
cargo r --example sprite_color --no-default-features --features="bevy_sprite bevy/bevy_winit bevy/bevy_picking"
cargo r --example colormaterial_color --no-default-features --features="bevy_sprite bevy/bevy_winit bevy/bevy_picking"
# bevy_ui
cargo r --example ui_position --no-default-features --features="bevy_sprite bevy_ui bevy/bevy_winit bevy/bevy_picking"
# bevy_text
cargo r --example text_color --no-default-features --features="bevy_text bevy_ui bevy/bevy_winit bevy/bevy_picking"
