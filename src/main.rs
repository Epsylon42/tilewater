#![feature(fn_traits)]
#![feature(unboxed_closures)]

use bevy::prelude::*;

mod mouse;
mod tiles;

use tiles::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(mouse::MousePosPlugin)
        .add_plugin(tiles::TilesPlugin)
        .add_plugin(tiles::TilesEditorPlugin)
        .add_plugin(AppPlugin)
        .run();
}

struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init.system());
    }
}

fn init(
    cmd: &mut Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn(Camera2dBundle::default());

    let solid_handle = asset_server.load("solid.png");
    let atlas = TextureAtlas::from_grid(solid_handle, Vec2::new(16.0, 16.0), 2, 1);
    let solid_handle = atlases.add(atlas);

    let liquid_handle = asset_server.load("liquid.png");
    let liquid_handle = materials.add(ColorMaterial {
        color: Color::WHITE,
        texture: Some(liquid_handle),
    });

    let mut tiles = WorldTiles::new(solid_handle, liquid_handle);
    let solid = &mut tiles.solid;

    for column in 0..10 {
        *solid.tiles.get_or_create(&[column, 5]) = OptTileIndex::from_index(1);
    }

    for row in 0..5 {
        for column in 0..10 {
            *solid.tiles.get_or_create(&[column, row]) = OptTileIndex::from_index(0);
        }
    }

    cmd.spawn(tiles);
}
