#[macro_use]
extern crate criterion;

use bevy::prelude::*;
use bevy_tweening::lens::*;
use criterion::{black_box, Criterion};

fn text_color_lens(c: &mut Criterion) {
    let mut lens = TextColorLens {
        start: Color::RED,
        end: Color::BLUE,
        section: 0,
    };
    let mut text = Text::from_section(
        "test".to_string(),
        TextStyle {
            font: Default::default(),
            font_size: 60.0,
            color: Color::WHITE,
        },
    )
    .with_alignment(TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    });
    c.bench_function("TextColorLens", |b| {
        b.iter(|| lens.lerp(&mut text, black_box(0.3)))
    });
}

fn transform_position_lens(c: &mut Criterion) {
    let mut lens = TransformPositionLens {
        start: Vec3::ZERO,
        end: Vec3::ONE,
    };
    let mut transform = Transform::identity();
    c.bench_function("TransformPositionLens", |b| {
        b.iter(|| lens.lerp(&mut transform, black_box(0.3)))
    });
}

fn transform_rotation_lens(c: &mut Criterion) {
    let mut lens = TransformRotationLens {
        start: Quat::IDENTITY,
        end: Quat::from_rotation_x(72.0_f32.to_radians()),
    };
    let mut transform = Transform::identity();
    c.bench_function("TransformRotationLens", |b| {
        b.iter(|| lens.lerp(&mut transform, black_box(0.3)))
    });
}

fn transform_scale_lens(c: &mut Criterion) {
    let mut lens = TransformScaleLens {
        start: Vec3::ONE,
        end: Vec3::new(1.5, 2.0, 3.0),
    };
    let mut transform = Transform::identity();
    c.bench_function("TransformScaleLens", |b| {
        b.iter(|| lens.lerp(&mut transform, black_box(0.3)))
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
