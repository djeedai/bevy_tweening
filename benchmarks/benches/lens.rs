#[macro_use]
extern crate criterion;

use bevy::{
    color::palettes::css::{BLUE, RED},
    ecs::{change_detection::MaybeLocation, component::Tick},
    prelude::*,
};
use bevy_tweening::{lens::*, ComponentTarget};
use criterion::{black_box, Criterion};

fn text_color_lens(c: &mut Criterion) {
    let mut lens = TextColorLens {
        start: RED.into(),
        end: BLUE.into(),
    };
    let mut text_color = TextColor::default();
    let mut added = Tick::new(0);
    let mut last_changed = Tick::new(0);
    let mut caller = MaybeLocation::caller();
    let mut target = ComponentTarget::new(Mut::new(
        &mut text_color,
        &mut added,
        &mut last_changed,
        Tick::new(0),
        Tick::new(0),
        caller.as_mut(),
    ));
    c.bench_function("TextColorLens", |b| {
        b.iter(|| lens.lerp(&mut target, black_box(0.3)))
    });
}

fn transform_position_lens(c: &mut Criterion) {
    let mut lens = TransformPositionLens {
        start: Vec3::ZERO,
        end: Vec3::ONE,
    };
    let mut transform = Transform::IDENTITY;
    let mut added = Tick::new(0);
    let mut last_changed = Tick::new(0);
    let mut caller = MaybeLocation::caller();
    let mut target = ComponentTarget::new(Mut::new(
        &mut transform,
        &mut added,
        &mut last_changed,
        Tick::new(0),
        Tick::new(0),
        caller.as_mut(),
    ));
    c.bench_function("TransformPositionLens", |b| {
        b.iter(|| lens.lerp(&mut target, black_box(0.3)))
    });
}

fn transform_rotation_lens(c: &mut Criterion) {
    let mut lens = TransformRotationLens {
        start: Quat::IDENTITY,
        end: Quat::from_rotation_x(72.0_f32.to_radians()),
    };
    let mut transform = Transform::IDENTITY;
    let mut added = Tick::new(0);
    let mut last_changed = Tick::new(0);
    let mut caller = MaybeLocation::caller();
    let mut target = ComponentTarget::new(Mut::new(
        &mut transform,
        &mut added,
        &mut last_changed,
        Tick::new(0),
        Tick::new(0),
        caller.as_mut(),
    ));
    c.bench_function("TransformRotationLens", |b| {
        b.iter(|| lens.lerp(&mut target, black_box(0.3)))
    });
}

fn transform_scale_lens(c: &mut Criterion) {
    let mut lens = TransformScaleLens {
        start: Vec3::ONE,
        end: Vec3::new(1.5, 2.0, 3.0),
    };
    let mut transform = Transform::IDENTITY;
    let mut added = Tick::new(0);
    let mut last_changed = Tick::new(0);
    let mut caller = MaybeLocation::caller();
    let mut target = ComponentTarget::new(Mut::new(
        &mut transform,
        &mut added,
        &mut last_changed,
        Tick::new(0),
        Tick::new(0),
        caller.as_mut(),
    ));
    c.bench_function("TransformScaleLens", |b| {
        b.iter(|| lens.lerp(&mut target, black_box(0.3)))
    });
}

criterion_group!(
    benches,
    text_color_lens,
    transform_position_lens,
    transform_rotation_lens,
    transform_scale_lens
);
criterion_main!(benches);
