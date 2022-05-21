#![feature(fn_traits)]
#![feature(unboxed_closures)]

use bevy::prelude::*;

mod mouse;
mod tiles;

use tiles::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(mouse::MousePosPlugin)
        .add_plugin(tiles::TilesPlugin)
        .add_plugin(tiles::TilesEditorPlugin)
        .add_startup_system(init)
        .run();
}

fn init(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    cmd.spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());

    let solid_handle = asset_server.load("solid.png");
    let atlas = TextureAtlas::from_grid(solid_handle, Vec2::new(16.0, 16.0), 2, 1);
    let solid_handle = atlases.add(atlas);

    let liquid_handle: Handle<Image> = asset_server.load("liquid.png");

    let font_handle = asset_server.load("font.ttf");

    let mut tiles = WorldTiles::new(solid_handle, liquid_handle, font_handle);
    let solid = &mut tiles.solid;

    for column in 0..10 {
        *solid.tiles.get_or_create(&[column, 5]) = OptTileIndex::from_index(1);
    }

    for row in 0..5 {
        for column in 0..10 {
            *solid.tiles.get_or_create(&[column, row]) = OptTileIndex::from_index(0);
        }
    }

    cmd.spawn().insert_bundle(tiles);
}
