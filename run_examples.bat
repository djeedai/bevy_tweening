@echo on
echo Run all examples
REM Default
cargo r --example menu --no-default-features --features="bevy/bevy_winit"
cargo r --example transform_translation --no-default-features --features="bevy/bevy_winit"
cargo r --example transform_rotation --no-default-features --features="bevy/bevy_winit"
cargo r --example sequence --no-default-features --features="bevy/bevy_winit"
REM bevy_sprite
cargo r --example sprite_color --no-default-features --features="bevy_sprite bevy/bevy_winit"
REM bevy_ui
cargo r --example ui_position --no-default-features --features="bevy_ui bevy/bevy_winit"
REM bevy_text
cargo r --example text_color --no-default-features --features="bevy_text bevy/bevy_winit"
REM bevy_sprite + bevy_asset
cargo r --example colormaterial_color --no-default-features --features="bevy_asset bevy_sprite bevy/bevy_winit"