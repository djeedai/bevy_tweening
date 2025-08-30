@echo on
echo Run all examples
REM Default
cargo r --example menu --no-default-features --features="bevy_ui bevy_text bevy/bevy_winit bevy/bevy_picking"
cargo r --example transform_translation --no-default-features --features="bevy_sprite bevy/bevy_winit bevy/bevy_picking"
cargo r --example transform_rotation --no-default-features --features="bevy_sprite bevy/bevy_winit bevy/bevy_picking"
cargo r --example sequence --no-default-features --features="bevy_sprite bevy_text bevy/bevy_winit bevy/bevy_picking"
cargo r --example ambient_light --no-default-features --features="bevy_ui bevy_text bevy/bevy_winit bevy/bevy_picking bevy/bevy_pbr bevy/hdr bevy/tonemapping_luts"
cargo r --example follow --no-default-features --features="bevy_sprite bevy_text bevy/bevy_winit bevy/bevy_picking"
REM bevy_sprite
cargo r --example sprite_color --no-default-features --features="bevy_sprite bevy/bevy_winit bevy/bevy_picking"
cargo r --example colormaterial_color --no-default-features --features="bevy_sprite bevy/bevy_winit bevy/bevy_picking"
REM bevy_ui
cargo r --example ui_position --no-default-features --features="bevy_sprite bevy_ui bevy/bevy_winit bevy/bevy_picking"
REM bevy_text
cargo r --example text_color --no-default-features --features="bevy_text bevy_ui bevy/bevy_winit bevy/bevy_picking"
