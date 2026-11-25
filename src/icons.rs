//! Icons, drawn into small white-on-transparent textures (the UI font has no arrow or dice glyphs).

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

const SIZE: usize = 64;

#[derive(Resource)]
pub struct Icons {
    pub undo: Handle<Image>,
    pub redo: Handle<Image>,
    pub dice: Handle<Image>,
}

pub(crate) fn make_icons(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let undo_path = undo_path();
    let redo_path: Vec<Vec2> = undo_path.iter().map(|point| Vec2::new(SIZE as f32 - point.x, point.y)).collect();
    let undo_head = [Vec2::new(10.0, 22.0), Vec2::new(26.0, 10.0), Vec2::new(26.0, 34.0)];
    let redo_head = undo_head.map(|point| Vec2::new(SIZE as f32 - point.x, point.y));
    commands.insert_resource(Icons {
        undo: images.add(render(|point| stroke(point, &undo_path, 3.0).max(triangle(point, undo_head)))),
        redo: images.add(render(|point| stroke(point, &redo_path, 3.0).max(triangle(point, redo_head)))),
        dice: images.add(render(dice)),
    });
}

/// The top stroke, curving round on the right and back underneath.
fn undo_path() -> Vec<Vec2> {
    let mut path = vec![Vec2::new(18.0, 22.0), Vec2::new(40.0, 22.0)];
    let (center, radius) = (Vec2::new(40.0, 37.0), 15.0);
    path.extend((1..16).map(|step| {
        let angle = -core::f32::consts::FRAC_PI_2 + core::f32::consts::PI * step as f32 / 16.0;
        center + Vec2::new(angle.cos(), angle.sin()) * radius
    }));
    path.extend([Vec2::new(40.0, 52.0), Vec2::new(22.0, 52.0)]);
    path
}

fn dice(point: Vec2) -> f32 {
    let local = (point - Vec2::splat(SIZE as f32 / 2.0)).abs();
    let (half, corner) = (21.0, 8.0);
    let outside = (local - Vec2::splat(half - corner)).max(Vec2::ZERO).length() - corner;
    let outline = coverage((outside.abs()) - 2.0);
    let dots = [(-11.0, -11.0), (11.0, -11.0), (0.0, 0.0), (-11.0, 11.0), (11.0, 11.0)]
        .into_iter()
        .map(|(x, y)| coverage((point - Vec2::new(32.0 + x, 32.0 + y)).length() - 3.5))
        .fold(0.0, f32::max);
    outline.max(dots)
}

/// 1 inside, 0 outside, with a pixel of smoothing; `distance` is signed (negative inside).
fn coverage(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

fn stroke(point: Vec2, path: &[Vec2], half_width: f32) -> f32 {
    let distance = path
        .windows(2)
        .map(|segment| {
            let (start, end) = (segment[0], segment[1]);
            let along = ((point - start).dot(end - start) / (end - start).length_squared()).clamp(0.0, 1.0);
            point.distance(start + (end - start) * along)
        })
        .fold(f32::MAX, f32::min);
    coverage(distance - half_width)
}

fn triangle(point: Vec2, corners: [Vec2; 3]) -> f32 {
    let sign = |a: Vec2, b: Vec2| (point.x - b.x) * (a.y - b.y) - (a.x - b.x) * (point.y - b.y);
    let (first, second, third) = (sign(corners[0], corners[1]), sign(corners[1], corners[2]), sign(corners[2], corners[0]));
    let negative = first < 0.0 || second < 0.0 || third < 0.0;
    let positive = first > 0.0 || second > 0.0 || third > 0.0;
    if negative && positive { 0.0 } else { 1.0 }
}

fn render(shape: impl Fn(Vec2) -> f32) -> Image {
    let mut data = vec![255u8; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            data[(y * SIZE + x) * 4 + 3] = (shape(Vec2::new(x as f32 + 0.5, y as f32 + 0.5)) * 255.0) as u8;
        }
    }
    Image::new(
        Extent3d { width: SIZE as u32, height: SIZE as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
