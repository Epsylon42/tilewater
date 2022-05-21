use bevy::prelude::*;

use std::num::NonZeroU32;

use crate::mouse::MousePos;

mod generic_tiles;
mod liquid;
mod sync;

use generic_tiles::*;
use liquid::*;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct OptTileIndex(Option<NonZeroU32>);

impl OptTileIndex {
    pub fn empty() -> Self {
        OptTileIndex(None)
    }

    pub fn from_index(index: u32) -> Self {
        OptTileIndex(NonZeroU32::new(index + 1))
    }

    pub fn get_index(&self) -> Option<u32> {
        self.0.map(|x| x.get() - 1)
    }
}

#[derive(Component)]
pub struct SolidTiles {
    pub tiles: GenericTiles<i32, OptTileIndex>,
    pub atlas: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct LiquidTiles {
    pub tiles: GenericTiles<i32, LiquidTile>,
    pub image: Handle<Image>,
    pub font: Handle<Font>,
}

#[derive(Bundle)]
pub struct WorldTiles {
    pub solid: SolidTiles,
    pub liquid: LiquidTiles,
    pub transform: Transform,
}

impl WorldTiles {
    pub fn new(
        solid_atlas: Handle<TextureAtlas>,
        liquid_material: Handle<Image>,
        liquid_font: Handle<Font>,
    ) -> Self {
        WorldTiles {
            solid: SolidTiles {
                tiles: GenericTiles::new(16),
                atlas: solid_atlas,
            },
            liquid: LiquidTiles {
                tiles: GenericTiles::new(16),
                image: liquid_material,
                font: liquid_font,
            },
            transform: Default::default(),
        }
    }
}

fn liquid_sim(
    mut enabled: Local<bool>,
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&SolidTiles, &mut LiquidTiles)>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::Space) {
        *enabled = !*enabled;
        eprintln!(
            "liquid simulation {}",
            if *enabled { "enabled" } else { "disabled" }
        );
    }

    if *enabled || keys.just_pressed(KeyCode::S) {
        for (solid, mut liquid) in query.iter_mut() {
            liquid.tiles.step(&solid.tiles, 0.1);
        }
    }
}

fn clear_modified_solid(mut solid: Query<&mut SolidTiles>) {
    for mut solid in solid.iter_mut() {
        solid.tiles.clear_modified();
    }
}

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        let mut tiles_stage = SystemStage::single_threaded();
        tiles_stage
            .add_system(liquid_sim)
            .add_system(sync::tiles_sync::<SolidTiles>)
            .add_system(sync::tiles_sync::<LiquidTiles>)
            .add_system(clear_modified_solid);

        app.insert_resource(bevy_immediate::ImmediateRenderSettings {
            tile_size: Vec2::splat(16.0),
            ..default()
        })
        .add_plugin(bevy_immediate::ImmediateRenderPlugin)
        .add_stage("tiles", tiles_stage);
    }
}

#[derive(Default)]
struct EditorState {
    liquid: bool,
}

fn tiles_editor(
    mut editor_state: Local<EditorState>,
    keys: Res<Input<KeyCode>>,
    mouse_keys: Res<Input<MouseButton>>,
    mouse_pos: Res<MousePos>,
    mut solid: Query<&mut SolidTiles>,
    mut liquid: Query<&mut LiquidTiles>,
) {
    if keys.just_pressed(KeyCode::L) {
        editor_state.liquid = !editor_state.liquid;
        eprintln!(
            "liquid {}",
            if editor_state.liquid {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    if keys.just_pressed(KeyCode::C) {
        eprintln!("clear");
        for mut solid in solid.iter_mut() {
            solid.tiles.clear();
        }
        for mut liquid in liquid.iter_mut() {
            liquid.tiles.clear();
        }
    }

    //if !mouse_keys.pressed(MouseButton::Left) && !mouse_keys.pressed(MouseButton::Right) {
    //return;
    //}

    let pos = mouse_pos.get_world() / 16.0;
    let pos = [pos.x.round() as i32, pos.y.round() as i32];

    if editor_state.liquid {
        for mut liquid in liquid.iter_mut() {
            let tile = liquid.tiles.get_or_create(&pos);
            if mouse_keys.pressed(MouseButton::Left) {
                *tile = LiquidTile::new(1.0);
            }
            if mouse_keys.pressed(MouseButton::Right) {
                *tile = LiquidTile::new(0.0);
            }
            if keys.pressed(KeyCode::M) {
                *tile = LiquidTile::new(99.0);
            }
        }
    } else {
        for mut solid in solid.iter_mut() {
            let tile = solid.tiles.get_or_create(&pos);
            if mouse_keys.pressed(MouseButton::Left) {
                *tile = OptTileIndex::from_index(0);
            }
            if mouse_keys.pressed(MouseButton::Right) {
                *tile = OptTileIndex::empty();
            }
        }
    }
}

pub struct TilesEditorPlugin;

impl Plugin for TilesEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(tiles_editor);
    }
}
