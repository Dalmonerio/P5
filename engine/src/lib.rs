use bevy::{
    prelude::*,
    core_pipeline::clear_color::ClearColorConfig,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    App::new()
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
