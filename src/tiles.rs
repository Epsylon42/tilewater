use bevy::prelude::*;

use std::num::NonZeroU32;

use crate::mouse::MousePos;

mod generic_tiles;
mod liquid;

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

pub struct SolidTiles {
    pub tiles: GenericTiles<i32, OptTileIndex>,
    pub atlas: Handle<TextureAtlas>,
}

pub struct LiquidTiles {
    pub tiles: GenericTiles<i32, LiquidTile>,
    pub material: Handle<ColorMaterial>,
}

#[derive(Bundle)]
pub struct WorldTiles {
    pub solid: SolidTiles,
    pub liquid: LiquidTiles,
    pub transform: Transform,
}

impl WorldTiles {
    pub fn new(solid_atlas: Handle<TextureAtlas>, liquid_material: Handle<ColorMaterial>) -> Self {
        WorldTiles {
            solid: SolidTiles {
                tiles: GenericTiles::new(16),
                atlas: solid_atlas,
            },
            liquid: LiquidTiles {
                tiles: GenericTiles::new(16),
                material: liquid_material,
            },
            transform: Default::default(),
        }
    }
}

fn clean_old_tiles(
    cmd: &mut Commands,
    tiles: Query<Option<&Children>, Or<(With<SolidTiles>, With<LiquidTiles>)>>,
) {
    for children in tiles.iter() {
        if let Some(children) = children {
            for child in children.iter() {
                cmd.despawn(*child);
            }
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

    if *enabled {
        for (solid, mut liquid) in query.iter_mut() {
            liquid.tiles.step(&solid.tiles, time.delta_seconds());
        }
    }
}

fn solid_sync(cmd: &mut Commands, solid: Query<(Entity, &SolidTiles)>) {
    for (this, solid) in solid.iter() {
        cmd.set_current_entity(this);
        cmd.with_children(|cmd| {
            for (chunk_coord, chunk) in solid.tiles.indexed_chunks() {
                for (inner_coord, tile) in chunk.indexed_tiles() {
                    let [x, y] = solid.tiles.combine_coord(chunk_coord, &inner_coord);
                    let [x, y] = [x as f32 * 16.0, y as f32 * 16.0];
                    if let Some(index) = tile.get_index() {
                        cmd.spawn(SpriteSheetBundle {
                            sprite: TextureAtlasSprite {
                                color: Color::WHITE,
                                index,
                            },
                            global_transform: GlobalTransform::from_translation(Vec3::new(
                                x, y, 0.0,
                            )),
                            texture_atlas: solid.atlas.clone(),
                            ..Default::default()
                        });
                    }
                }
            }
        });
    }
}

fn liquid_sync(
    cmd: &mut Commands,
    liquid: Query<(Entity, &LiquidTiles)>,
    asset_server: Res<AssetServer>,
) {
    for (this, liquid) in liquid.iter() {
        cmd.set_current_entity(this);
        cmd.with_children(|cmd| {
            for (chunk_coord, chunk) in liquid.tiles.indexed_chunks() {
                for (point, tile) in chunk.indexed_tiles() {
                    let [x, y] = liquid.tiles.combine_coord(chunk_coord, &point);
                    let [x, y] = [x as f32 * 16.0, y as f32 * 16.0];

                    if !tile.is_empty() {
                        cmd.spawn(SpriteBundle {
                            material: liquid.material.clone(),
                            ..Default::default()
                        })
                        .with_bundle(Text2dBundle {
                            text: Text {
                                value: tile.to_string(),
                                font: asset_server.load("font.ttf"),
                                style: TextStyle {
                                    font_size: 10.0,
                                    color: Color::WHITE,
                                    alignment: TextAlignment {
                                        vertical: VerticalAlign::Center,
                                        horizontal: HorizontalAlign::Center,
                                    },
                                },
                            },
                            global_transform: GlobalTransform::from_translation(Vec3::new(
                                x, y, 0.0,
                            )),
                            ..Default::default()
                        });
                    }
                }
            }
        });
    }
}

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(liquid_sim.system())
            .add_system(clean_old_tiles.system())
            .add_system(solid_sync.system())
            .add_system(liquid_sync.system());
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

    if !mouse_keys.pressed(MouseButton::Left) && !mouse_keys.pressed(MouseButton::Right) {
        return;
    }

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
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(tiles_editor.system());
    }
}
