/*! Geometry convenience functions */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */
use bevy::math::{ Quat, Vec3 };
use bevy_rapier2d::na::{ Point2, Rotation2, UnitComplex, Vector2 };
use bevy_rapier2d::rapier::math::{ Isometry, Translation, Vector };
use std::cmp::Ordering::Equal;


pub fn angle_from(position: &Isometry<f32>, target: &Point2<f32>) -> f32 {
    let point: Point2<f32> = position.inverse_transform_point(target);
    Rotation2::rotation_between(
        &Vector2::new(0.0, 1.0),
        &Vector2::new(point.x, point.y),
    ).angle()
}

pub fn get_nearest(position: &Point2<f32>, others: &[Point2<f32>]) -> Option<Point2<f32>> {
    others.iter()
        .map(|p| (p, (p - position).norm()))
        .min_by(|(_, norm), (_, norm2)| norm.partial_cmp(norm2).unwrap_or(Equal))
        .map(|(p, _norm)| p.clone())
}

pub fn into_isometry_2d(translation: Vec3, rotation: Quat) -> Isometry<f32> {
    let (axis, angle) = rotation.to_axis_angle();
    let angle = match axis.z > 0.0 {
        true => angle,
        false => -angle,
    };
        
    Isometry::from_parts(
        Translation::from(Vector::new(translation.x, translation.y)),
        UnitComplex::new(angle),
    )
}
