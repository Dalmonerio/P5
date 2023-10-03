use bevy::{
    prelude::*,
    core_pipeline::clear_color::ClearColorConfig,
    ecs::system::SystemState,
};
use bevy_asset_loader::prelude::*;

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

#[derive(Resource)]
struct Atlas(Handle<TextureAtlas>);

impl FromWorld for Atlas {
    fn from_world(world: &mut World) -> Self {
        let (mut sprites, mut images, mut atlases) = SystemState::<(
            ResMut<Sprites>,
            ResMut<Assets<Image>>,
            ResMut<Assets<TextureAtlas>>,
        )>
            ::new(world).get_mut(world);

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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_state::<EngineState>()

        .insert_resource(Msaa::Sample4)
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
        .run();
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::hex("#0B0518FF").unwrap()),
        },
        ..default()
    });
}
