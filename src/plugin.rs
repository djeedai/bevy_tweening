use bevy::{asset::Asset, ecs::component::Component, prelude::*};

use crate::{Animator, AnimatorState, AssetAnimator, TweeningType};

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
        if animator.state == AnimatorState::Playing {
            animator.timer.tick(time.delta());
        }
        if animator.paused {
            if animator.timer.just_finished() {
                match animator.tweening_type {
                    TweeningType::Once { duration } => {
                        animator.timer.set_duration(duration);
                    }
                    TweeningType::Loop { duration, .. } => {
                        animator.timer.set_duration(duration);
                    }
                    TweeningType::PingPong { duration, .. } => {
                        animator.timer.set_duration(duration);
                    }
                }
                animator.timer.reset();
                animator.paused = false;
            }
        } else {
            if animator.timer.duration().as_secs_f32() != 0. {
                let progress = animator.progress();
                let factor = animator.ease_function.sample(progress);
                animator.apply(target, factor);
            }
            if animator.timer.finished() {
                match animator.tweening_type {
                    TweeningType::Once { .. } => {
                        //commands.entity(entity).remove::<Animator>();
                    }
                    TweeningType::Loop { pause, .. } => {
                        if let Some(pause) = pause {
                            animator.timer.set_duration(pause);
                            animator.paused = true;
                        }
                        animator.timer.reset();
                    }
                    TweeningType::PingPong { pause, .. } => {
                        if let Some(pause) = pause {
                            animator.timer.set_duration(pause);
                            animator.paused = true;
                        }
                        animator.timer.reset();
                        animator.direction = !animator.direction;
                    }
                }
            }
        }
    }
}

pub fn asset_animator_system<T: Asset>(
    time: Res<Time>,
    mut assets: ResMut<Assets<T>>,
    mut query: Query<&mut AssetAnimator<T>>,
) {
    for ref mut animator in query.iter_mut() {
        if animator.state == AnimatorState::Playing {
            animator.timer.tick(time.delta());
        }
        if animator.paused {
            if animator.timer.just_finished() {
                match animator.tweening_type {
                    TweeningType::Once { duration } => {
                        animator.timer.set_duration(duration);
                    }
                    TweeningType::Loop { duration, .. } => {
                        animator.timer.set_duration(duration);
                    }
                    TweeningType::PingPong { duration, .. } => {
                        animator.timer.set_duration(duration);
                    }
                }
                animator.timer.reset();
                animator.paused = false;
            }
        } else {
            if animator.timer.duration().as_secs_f32() != 0. {
                let progress = animator.progress();
                let factor = animator.ease_function.sample(progress);
                if let Some(target) = assets.get_mut(animator.handle()) {
                    animator.apply(target, factor);
                }
            }
            if animator.timer.finished() {
                match animator.tweening_type {
                    TweeningType::Once { .. } => {
                        //commands.entity(entity).remove::<Animator>();
                    }
                    TweeningType::Loop { pause, .. } => {
                        if let Some(pause) = pause {
                            animator.timer.set_duration(pause);
                            animator.paused = true;
                        }
                        animator.timer.reset();
                    }
                    TweeningType::PingPong { pause, .. } => {
                        if let Some(pause) = pause {
                            animator.timer.set_duration(pause);
                            animator.paused = true;
                        }
                        animator.timer.reset();
                        animator.direction = !animator.direction;
                    }
                }
            }
        }
    }
}
