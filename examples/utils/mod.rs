#![allow(unused)]

use bevy::prelude::*;

pub fn close_on_esc(mut ev_app_exit: EventWriter<AppExit>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        ev_app_exit.write(AppExit::Success);
    }
}
