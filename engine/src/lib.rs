use bevy::{
    prelude::*,
    core_pipeline::clear_color::ClearColorConfig,
    ecs::system::SystemState,
    window::{
        PrimaryWindow, PresentMode,
    },
};
use bevy_asset_loader::prelude::*;
use rand::prelude::*;
use std::f32::consts::PI;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum EngineState {
    #[default]
    Loading,
    Running,
}

#[derive(AssetCollection, Resource)]
struct Sprites {
    #[asset(path = "bottle.png")]
    bottle: Handle<Image>,
    #[asset(path = "circle.png")]
    circle: Handle<Image>,
}

#[derive(Resource, Deref)]
struct Atlas(Handle<TextureAtlas>);
impl Atlas {
    fn index(&self, atlases: &Assets<TextureAtlas>, sprite: &Handle<Image>) -> usize {
        atlases.get(&self.0).unwrap().get_texture_index(sprite).unwrap()
    }
}

impl FromWorld for Atlas {
    fn from_world(world: &mut World) -> Self {
        let (mut sprites, mut images, mut atlases) = SystemState::<(
            ResMut<Sprites>,
            ResMut<Assets<Image>>,
            ResMut<Assets<TextureAtlas>>,
        )>::new(world).get_mut(world);

        let mut builder = TextureAtlasBuilder::default();
        let mut add = |sprite: &mut Handle<Image>| {
            let handle = std::mem::replace(sprite, sprite.clone_weak());
            builder.add_texture(handle, images.get(sprite).unwrap());
        };

        add(&mut sprites.bottle);
        add(&mut sprites.circle);

        let atlas = builder.finish(&mut images).unwrap();
        Self(atlases.add(atlas))
    }
}

#[derive(Component, Copy, Clone)]
struct Timed {
    time: f32,
    lifetime: f32,
}

impl Timed {
    fn new(lifetime: f32) -> Self {
        Self { time: 0.0, lifetime, }
    }

    fn fin(self) -> f32 {
        return self.time / self.lifetime;
    }
}

#[derive(Component, Copy, Clone)]
struct Particle {
    deviate: Vec2,
    alpha: f32,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_state::<EngineState>()

        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins
            .set(AssetPlugin {
                #[cfg(target_arch = "wasm32")]
                asset_folder: "engine/assets".into(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bg-canvas".into()),
                    prevent_default_event_handling: false,
                    fit_canvas_to_parent: true,
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            })
        )

        .add_loading_state(LoadingState::new(EngineState::Loading)
            .continue_to_state(EngineState::Running)
        )
        .add_collection_to_loading_state::<_, Sprites>(EngineState::Loading)
        .init_resource_after_loading_state::<_, Atlas>(EngineState::Loading)

        .add_systems(Startup, init)
        .add_systems(Update, (spawn_particles, update_particles).run_if(in_state(EngineState::Running)))
        .add_systems(Update, timed_accumulate.run_if(in_state(EngineState::Running)))
        .add_systems(PostUpdate, timed_check.run_if(in_state(EngineState::Running)))

        .run();
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::hex("#0B0518FF").unwrap()),
        },
        transform: Transform::from_xyz(0.0, 0.0, 100.0),
        ..default()
    });
}

fn spawn_particles(
    mut commands: Commands,
    time: Res<Time>, window: Query<&Window, With<PrimaryWindow>>,
    sprites: Res<Sprites>, atlas: Res<Atlas>, atlases: Res<Assets<TextureAtlas>>,
) {
    let Ok(window) = window.get_single() else { return };

    let mut rng = thread_rng();
    let chance = (window.width() * window.height()) / (1920.0 * 1080.0);
    if rng.gen::<f32>() * time.delta_seconds() <= chance * 0.4 {
        commands.spawn((
            Timed::new(2.4),
            Particle {
                deviate: Vec2::from_angle(rng.gen::<f32>() * 2.0 * PI) * rng.gen_range(16.0f32..64.0f32),
                alpha: rng.gen_range(0.3f32..0.7f32),
            },
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: atlas.index(&atlases, &sprites.circle),
                    custom_size: Some(Vec2::splat(rng.gen_range(24.0f32..84.0f32))),
                    color: Color::hex("#251240").unwrap().with_a(0.),
                    ..default()
                },
                texture_atlas: atlas.clone_weak(),
                transform: Transform::from_xyz(
                    (rng.gen::<f32>() - 0.5) * window.width(),
                    (rng.gen::<f32>() - 0.5) * window.height(),
                    0.0,
                ),
                ..default()
            },
        ));
    }
}

fn update_particles(
    time: Res<Time>,
    mut particles: Query<(&Particle, &Timed, &mut Transform, &mut TextureAtlasSprite)>,
) {
    let delta = time.delta_seconds();
    for (&particle, &timed, mut trns, mut sprite) in &mut particles {
        let f = timed.fin();
        sprite.color.set_a((1.0 - (f * 2.0 - 1.0).abs()) * particle.alpha);
        trns.translation += (particle.deviate * delta).extend(0.0);
    }
}

fn timed_accumulate(time: Res<Time>, mut timed: Query<&mut Timed>) {
    let delta = time.delta_seconds();
    for mut timed in &mut timed {
        timed.time = (timed.time + delta).min(timed.lifetime);
    }
}

fn timed_check(mut commands: Commands, timed: Query<(Entity, &Timed)>) {
    for (e, &timed) in &timed {
        if timed.time >= timed.lifetime {
            commands.entity(e).despawn();
        }
    }
}
