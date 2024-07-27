use bevy::{color::palettes::css, prelude::*};

use avian2d::{prelude as avian, prelude::*};
#[allow(unused_imports)]
use bevy_tnua::math::{AdjustPrecision, Vector2, Vector3};
use bevy_tnua::TnuaGhostPlatform;

use crate::level_mechanics::MovingPlatform;

use super::{LevelObject, PositionPlayer};

#[derive(PhysicsLayer)]
pub enum LayerNames {
    Player,
    FallThrough,
    PhaseThrough,
}

pub fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(PositionPlayer::from(Vec3::new(0.0, 2.0, 0.0)));

    let mut cmd = commands.spawn((LevelObject, Name::new("Floor")));
    cmd.insert(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(128.0, 0.5)),
            color: css::GRAY.into(),
            ..Default::default()
        },
        ..Default::default()
    });
    {
        cmd.insert(avian::RigidBody::Static);
        cmd.insert(avian::Collider::half_space(Vector2::Y));
    }

    for (name, [width, height], transform) in [
        (
            "Moderate Slope",
            [10.0, 0.1],
            Transform::from_xyz(7.0, 7.0, 0.0).with_rotation(Quat::from_rotation_z(0.6)),
        ),
        (
            "Steep Slope",
            [10.0, 0.1],
            Transform::from_xyz(14.0, 14.0, 0.0).with_rotation(Quat::from_rotation_z(1.0)),
        ),
        (
            "Box to Step on",
            [4.0, 2.0],
            Transform::from_xyz(-4.0, 1.0, 0.0),
        ),
        (
            "Floating Box",
            [6.0, 1.0],
            Transform::from_xyz(-10.0, 4.0, 0.0),
        ),
        (
            "Box to Crawl Under",
            [6.0, 1.0],
            Transform::from_xyz(-20.0, 2.6, 0.0),
        ),
    ] {
        let mut cmd = commands.spawn((LevelObject, Name::new(name)));
        cmd.insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(width, height)),
                color: css::GRAY.into(),
                ..Default::default()
            },
            transform,
            ..Default::default()
        });
        {
            cmd.insert(avian::RigidBody::Static);
            cmd.insert(avian::Collider::rectangle(
                width.adjust_precision(),
                height.adjust_precision(),
            ));
        }
    }

    // Fall-through platforms
    for (i, y) in [5.0, 7.5].into_iter().enumerate() {
        let mut cmd = commands.spawn((LevelObject, Name::new(format!("Fall Through #{}", i + 1))));
        cmd.insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(6.0, 0.5)),
                color: css::PINK.into(),
                ..Default::default()
            },
            transform: Transform::from_xyz(-20.0, y, -1.0),
            ..Default::default()
        });
        {
            cmd.insert(avian::RigidBody::Static);
            cmd.insert(avian::Collider::rectangle(6.0, 0.5));
            cmd.insert(CollisionLayers::new(
                [LayerNames::FallThrough],
                [LayerNames::FallThrough],
            ));
        }
        cmd.insert(TnuaGhostPlatform);
    }

    commands.spawn((
        LevelObject,
        Name::new("Collision Groups"),
        TransformBundle::from_transform(Transform::from_xyz(10.0, 2.0, 0.0)),
        (
            avian::RigidBody::Static,
            avian::Collider::circle(1.0),
            CollisionLayers::new([LayerNames::PhaseThrough], [LayerNames::PhaseThrough]),
        ),
    ));
    commands.spawn((
        LevelObject,
        Text2dBundle {
            text: Text::from_section(
                "collision\ngroups",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 72.0,
                    color: css::WHITE.into(),
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(10.0, 2.0, 1.0).with_scale(0.01 * Vec3::ONE),
            ..Default::default()
        },
    ));

    // #[cfg(feature = "rapier2d")]
    // {
    //     commands.spawn((
    //         LevelObject,
    //         Name::new("Solver Groups"),
    //         TransformBundle::from_transform(Transform::from_xyz(15.0, 2.0, 0.0)),
    //         rapier::Collider::ball(1.0),
    //         SolverGroups {
    //             memberships: Group::GROUP_1,
    //             filters: Group::GROUP_1,
    //         },
    //     ));
    //     commands.spawn((
    //         LevelObject,
    //         Text2dBundle {
    //             text: Text::from_section(
    //                 "solver\ngroups",
    //                 TextStyle {
    //                     font: asset_server.load("FiraSans-Bold.ttf"),
    //                     font_size: 72.0,
    //                     color: css::WHITE.into(),
    //                 },
    //             )
    //             .with_justify(JustifyText::Center),
    //             transform: Transform::from_xyz(15.0, 2.0, 1.0).with_scale(0.01 * Vec3::ONE),
    //             ..Default::default()
    //         },
    //     ));
    // }

    commands.spawn((
        LevelObject,
        Name::new("Sensor"),
        TransformBundle::from_transform(Transform::from_xyz(20.0, 2.0, 0.0)),
        (
            avian::RigidBody::Static,
            avian::Collider::circle(1.0),
            avian::Sensor,
        ),
    ));
    commands.spawn((
        LevelObject,
        Text2dBundle {
            text: Text::from_section(
                "sensor",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 72.0,
                    color: css::WHITE.into(),
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(20.0, 2.0, 1.0).with_scale(0.01 * Vec3::ONE),
            ..Default::default()
        },
    ));

    // spawn moving platform
    {
        let mut cmd = commands.spawn((LevelObject, Name::new("Moving Platform")));
        cmd.insert(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(4.0, 1.0)),
                color: css::BLUE.into(),
                ..Default::default()
            },
            transform: Transform::from_xyz(-4.0, 6.0, 0.0),
            ..Default::default()
        });
        {
            cmd.insert(avian::Collider::rectangle(4.0, 1.0));
            cmd.insert(avian::RigidBody::Kinematic);
        }
        cmd.insert(MovingPlatform::new(
            4.0,
            &[
                Vector3::new(-4.0, 6.0, 0.0),
                Vector3::new(-8.0, 6.0, 0.0),
                Vector3::new(-8.0, 10.0, 0.0),
                Vector3::new(-4.0, 10.0, 0.0),
            ],
        ));
    }
}
