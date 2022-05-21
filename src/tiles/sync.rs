use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use bevy_immediate::{ImmediateRenderObject, ImmediateRenderRequest};

use super::*;

pub trait Tile: Default + Send + Sync + 'static {
    fn needs_sprite(&self) -> bool;
}

pub trait Tilemap: Send + Sync + 'static {
    type Tile: Tile;
    fn storage(&self) -> &GenericTiles<i32, Self::Tile>;
    fn storage_mut(&mut self) -> &mut GenericTiles<i32, Self::Tile>;
    fn create_request(&self, tile: &Self::Tile) -> Vec<ImmediateRenderObject>;
}

pub fn tiles_sync<T: Tilemap + Component>(
    mut tiles: Query<&mut T>,
    mut requests: EventWriter<ImmediateRenderRequest>,
) {
    for tiles in tiles.iter_mut() {
        for (coord, tile) in tiles.storage().indexed_tiles() {
            if tile.needs_sprite() {
                requests.send_batch(tiles.create_request(tile).into_iter().enumerate().map(
                    |(i, obj)| ImmediateRenderRequest {
                        obj,
                        z: i as f32 * 0.1,
                        pos: IVec2::new(coord[0], coord[1]),
                    },
                ));
            }
        }
    }
}

impl Tile for OptTileIndex {
    fn needs_sprite(&self) -> bool {
        self.get_index().is_some()
    }
}

impl Tile for LiquidTile {
    fn needs_sprite(&self) -> bool {
        !self.is_empty()
    }
}

impl Tilemap for SolidTiles {
    type Tile = OptTileIndex;

    fn storage(&self) -> &GenericTiles<i32, Self::Tile> {
        &self.tiles
    }

    fn storage_mut(&mut self) -> &mut GenericTiles<i32, Self::Tile> {
        &mut self.tiles
    }

    fn create_request(&self, tile: &Self::Tile) -> Vec<ImmediateRenderObject> {
        vec![ImmediateRenderObject::SheetSprite {
            index: tile.get_index().unwrap() as usize,
            color: Color::WHITE,
            atlas: self.atlas.clone(),
        }]
    }
}

impl Tilemap for LiquidTiles {
    type Tile = LiquidTile;

    fn storage(&self) -> &GenericTiles<i32, Self::Tile> {
        &self.tiles
    }

    fn storage_mut(&mut self) -> &mut GenericTiles<i32, Self::Tile> {
        &mut self.tiles
    }

    fn create_request(&self, tile: &Self::Tile) -> Vec<ImmediateRenderObject> {
        vec![
            ImmediateRenderObject::Sprite {
                //color: Color::rgb((tile.velocity.x - tile.velocity.z), tile.velocity.y - tile.velocity.w, 0.0),
                //image: DEFAULT_IMAGE_HANDLE.typed(),
                color: Color::WHITE,
                image: self.image.clone(),
            },
            ImmediateRenderObject::Label(TextSection {
                value: tile.to_string(),
                style: TextStyle {
                    font: self.font.clone(),
                    font_size: 10.0,
                    color: Color::WHITE,
                },
            }),
        ]
    }
}
