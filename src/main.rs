// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod app_setup_options;
mod level_mechanics;
mod levels_setup;
mod systems;
mod ui;

use app_setup_options::{AppSetupConfiguration, ScheduleToUse};

use avian2d::{prelude as avian, prelude::*, schedule::PhysicsSchedule};

// use bevy::asset::AssetMetaCheck;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use bevy_tnua::builtins::TnuaBuiltinCrouch;
use bevy_tnua::control_helpers::{
    TnuaCrouchEnforcer, TnuaCrouchEnforcerPlugin, TnuaSimpleAirActionsCounter,
    TnuaSimpleFallThroughPlatformsHelper,
};
#[allow(unused_imports)]
use bevy_tnua::math::{float_consts, AsF32, Vector3};
use bevy_tnua::prelude::*;
use bevy_tnua::{TnuaGhostSensor, TnuaToggle};
use bevy_tnua_avian2d::*;

use systems::character_control::info_dumping::character_control_info_dumping_system;
use systems::character_control::platformer_control::{
    apply_platformer_controls, CharacterMotionConfigForPlatformerDemo, FallingThroughControlScheme,
};
use systems::character_control::Dimensionality;

use level_mechanics::LevelMechanicsPlugin;

use levels_setup::demo::LayerNames;
use levels_setup::level_switching::LevelSwitchingPlugin;
use levels_setup::IsPlayer;

use ui::component_alteration::CommandAlteringSelectors;
use ui::info::InfoSource;
use ui::plotting::PlotSource;
use ui::DemoInfoUpdateSystemSet;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    let app_setup_configuration = AppSetupConfiguration::from_environment();
    app.insert_resource(app_setup_configuration.clone());

    {
        app.add_plugins(PhysicsDebugPlugin::default());
        match app_setup_configuration.schedule_to_use {
            ScheduleToUse::Update => {
                app.add_plugins(PhysicsPlugins::default());
                // To use Tnua with avian2d, you need the `TnuaAvian2dPlugin` plugin from
                // bevy-tnua-avian2d.
                app.add_plugins(TnuaAvian2dPlugin::default());
            }
            ScheduleToUse::FixedUpdate => {
                app.add_plugins(PhysicsPlugins::new(FixedUpdate));
                app.add_plugins(TnuaAvian2dPlugin::new(FixedUpdate));
            }
            ScheduleToUse::PhysicsSchedule => {
                app.add_plugins(PhysicsPlugins::default());
                app.insert_resource(Time::new_with(Physics::fixed_hz(144.0)));
                app.add_plugins(TnuaAvian2dPlugin::new(PhysicsSchedule));
            }
        }
    }

    match app_setup_configuration.schedule_to_use {
        ScheduleToUse::Update => {
            // This is Tnua's main plugin.
            app.add_plugins(TnuaControllerPlugin::default());

            // This plugin supports `TnuaCrouchEnforcer`, which prevents the character from standing up
            // while obstructed by an obstacle.
            app.add_plugins(TnuaCrouchEnforcerPlugin::default());
        }
        ScheduleToUse::FixedUpdate => {
            app.add_plugins(TnuaControllerPlugin::new(FixedUpdate));
            app.add_plugins(TnuaCrouchEnforcerPlugin::new(FixedUpdate));
        }
        ScheduleToUse::PhysicsSchedule => {
            app.add_plugins(TnuaControllerPlugin::new(PhysicsSchedule));
            app.add_plugins(TnuaCrouchEnforcerPlugin::new(PhysicsSchedule));
        }
    }

    app.add_systems(
        Update,
        character_control_info_dumping_system.in_set(DemoInfoUpdateSystemSet),
    );
    app.add_plugins(ui::DemoUi::<CharacterMotionConfigForPlatformerDemo>::default());
    app.add_systems(Startup, setup_camera_and_lights);
    app.add_plugins({
        LevelSwitchingPlugin::new(app_setup_configuration.level_to_load.as_ref())
            .with("Default", levels_setup::demo::setup_level)
    });
    app.add_systems(Startup, setup_player);
    app.add_systems(
        match app_setup_configuration.schedule_to_use {
            ScheduleToUse::Update => Update.intern(),
            ScheduleToUse::FixedUpdate => FixedUpdate.intern(),
            ScheduleToUse::PhysicsSchedule => PhysicsSchedule.intern(),
        },
        apply_platformer_controls.in_set(TnuaUserControlsSystemSet),
    );
    app.add_plugins(LevelMechanicsPlugin);

    app.run();
    // App::new()
    //     .add_plugins(DefaultPlugins.set(AssetPlugin {
    //         // Wasm builds will check for meta files (that don't exist) if this isn't set.
    //         // This causes errors and even panics in web builds on itch.
    //         // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
    //         meta_check: AssetMetaCheck::Never,
    //         ..default()
    //     }))
    //     .add_systems(Startup, setup)
    //     .run();
}

// fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn(Camera2dBundle::default());
//     commands.spawn(SpriteBundle {
//         texture: asset_server.load("ducky.png"),
//         ..Default::default()
//     });
// }

fn setup_camera_and_lights(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 14.0, 30.0)
            .with_scale((0.05 * Vec2::ONE).extend(1.0))
            .looking_at(Vec3::new(0.0, 14.0, 0.0), Vec3::Y),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });
}

fn setup_player(mut commands: Commands) {
    let mut cmd = commands.spawn(IsPlayer);
    cmd.insert(TransformBundle::default());
    cmd.insert(VisibilityBundle::default());

    // The character entity must be configured as a dynamic rigid body of the physics backend.
    {
        cmd.insert(avian::RigidBody::Dynamic);
        cmd.insert(avian::Collider::capsule(0.5, 1.0));
        // Avian does not need an "IO" bundle.
    }

    // This bundle container `TnuaController` - the main interface of Tnua with the user code - as
    // well as the main components used as API between the main plugin and the physics backend
    // integration. These components (and the IO bundle, in case of backends that need one like
    // Rapier) are the only mandatory Tnua components - but this example will also add some
    // components used for more advanced features.
    //
    // Read examples/src/character_control_systems/platformer_control_systems.rs to see how
    // `TnuaController` is used in this example.
    cmd.insert(TnuaControllerBundle::default());

    cmd.insert(CharacterMotionConfigForPlatformerDemo {
        dimensionality: Dimensionality::Dim2,
        speed: 40.0,
        walk: TnuaBuiltinWalk {
            float_height: 2.0,
            max_slope: float_consts::FRAC_PI_4,
            ..Default::default()
        },
        actions_in_air: 1,
        jump: TnuaBuiltinJump {
            height: 4.0,
            ..Default::default()
        },
        crouch: TnuaBuiltinCrouch {
            float_offset: -0.9,
            ..Default::default()
        },
        dash_distance: 10.0,
        dash: Default::default(),
        one_way_platforms_min_proximity: 1.0,
        falling_through: FallingThroughControlScheme::SingleFall,
    });

    // An entity's Tnua behavior can be toggled individually with this component, if inserted.
    cmd.insert(TnuaToggle::default());
    cmd.insert({
        let command_altering_selectors = CommandAlteringSelectors::default()
            // By default Tnua uses a raycast, but this could be a problem if the character stands
            // just past the edge while part of its body is above the platform. To solve this, we
            // need to cast a shape - which is physics-engine specific. We set the shape using a
            // component.
            .with_combo(
                "Sensor Shape",
                1,
                &[
                    ("Point", |mut cmd| {
                        cmd.remove::<TnuaAvian2dSensorShape>();
                    }),
                    ("Flat (underfit)", |mut cmd| {
                        cmd.insert(TnuaAvian2dSensorShape(avian::Collider::rectangle(
                            0.99, 0.0,
                        )));
                    }),
                    ("Flat (exact)", |mut cmd| {
                        cmd.insert(TnuaAvian2dSensorShape(avian::Collider::rectangle(1.0, 0.0)));
                    }),
                    ("flat (overfit)", |mut cmd| {
                        cmd.insert(TnuaAvian2dSensorShape(avian::Collider::rectangle(
                            1.01, 0.0,
                        )));
                    }),
                    ("Ball (underfit)", |mut cmd| {
                        cmd.insert(TnuaAvian2dSensorShape(avian::Collider::circle(0.49)));
                    }),
                    ("Ball (exact)", |mut cmd| {
                        cmd.insert(TnuaAvian2dSensorShape(avian::Collider::circle(0.5)));
                    }),
                ],
            )
            .with_checkbox("Lock Tilt", false, |mut cmd, lock_tilt| {
                // Tnua will automatically apply angular impulses/forces to fix the tilt and make
                // the character stand upward, but it is also possible to just let the physics
                // engine prevent rotation (other than around the Y axis, for turning)
                if lock_tilt {
                    cmd.insert(avian::LockedAxes::new().lock_rotation());
                } else {
                    cmd.insert(avian::LockedAxes::new());
                }
            })
            .with_checkbox(
                "Phase Through Collision Groups",
                true,
                |mut cmd, use_collision_groups| {
                    let player_layers: LayerMask = if use_collision_groups {
                        [LayerNames::Player].into()
                    } else {
                        [LayerNames::Player, LayerNames::PhaseThrough].into()
                    };
                    cmd.insert(CollisionLayers::new(player_layers, player_layers));
                },
            );
        command_altering_selectors
    });

    // `TnuaCrouchEnforcer` can be used to prevent the character from standing up when obstructed.
    cmd.insert(TnuaCrouchEnforcer::new(0.5 * Vector3::Y, |cmd| {
        // It needs a sensor shape because it needs to do a shapecast upwards. Without a sensor shape
        // it'd do a raycast.
        cmd.insert(TnuaAvian2dSensorShape(avian::Collider::rectangle(1.0, 0.0)));
    }));

    // The ghost sensor is used for detecting ghost platforms - platforms configured in the physics
    // backend to not contact with the character (or detect the contact but not apply physical
    // forces based on it) and marked with the `TnuaGhostPlatform` component. These can then be
    // used as one-way platforms.
    cmd.insert(TnuaGhostSensor::default());

    // This helper is used to operate the ghost sensor and ghost platforms and implement
    // fall-through behavior where the player can intentionally fall through a one-way platform.
    cmd.insert(TnuaSimpleFallThroughPlatformsHelper::default());

    // This helper keeps track of air actions like jumps or air dashes.
    cmd.insert(TnuaSimpleAirActionsCounter::default());

    cmd.insert((
        ui::TrackedEntity("Player".to_owned()),
        PlotSource::default(),
        InfoSource::default(),
    ));
}
