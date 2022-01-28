use bevy::{asset::Asset, ecs::component::Component, prelude::*};

use crate::{Animator, AnimatorState, AssetAnimator};

/// Plugin to add systems related to tweening
#[derive(Debug, Clone, Copy)]
pub struct TweeningPlugin;

impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(component_animator_system::<Transform>)
            .add_system(component_animator_system::<Text>)
            .add_system(component_animator_system::<Style>)
            .add_system(component_animator_system::<Sprite>)
            .add_system(asset_animator_system::<ColorMaterial>);
    }
}

pub fn component_animator_system<T: Component>(
    time: Res<Time>,
    mut query: Query<(&mut T, &mut Animator<T>)>,
) {
    for (ref mut target, ref mut animator) in query.iter_mut() {
        if animator.state == AnimatorState::Paused {
            continue;
        }
        // Play all tracks in parallel
        for seq in &mut animator.tracks_mut().tracks {
            seq.tick(time.delta(), target);
        }
    }
}

pub fn asset_animator_system<T: Asset>(
    time: Res<Time>,
    mut assets: ResMut<Assets<T>>,
    mut query: Query<&mut AssetAnimator<T>>,
) {
    for ref mut animator in query.iter_mut() {
        if animator.state == AnimatorState::Paused {
            continue;
        }
        if let Some(target) = assets.get_mut(animator.handle()) {
            // Play all tracks in parallel
            for seq in &mut animator.tracks_mut().tracks {
                seq.tick(time.delta(), target);
            }
        }
    }
}
