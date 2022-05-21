use super::generic_tiles::*;
use super::OptTileIndex;
use bevy::prelude::Vec4;

#[derive(Default, Clone)]
pub struct LiquidTile {
    pub amount: f32,
    pub velocity: Vec4,
}

impl LiquidTile {
    pub fn new(amount: f32) -> Self {
        LiquidTile {
            amount,
            velocity: Vec4::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.amount <= 0.01
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Right,
    Down,
    Left,
    Up,
}

impl Direction {
    fn all() -> [Direction; 4] {
        [
            Direction::Right,
            Direction::Down,
            Direction::Left,
            Direction::Up,
        ]
        //[Direction::Down, Direction::Up]
    }

    fn offset(self) -> [i32; 2] {
        match self {
            Direction::Right => [1, 0],
            Direction::Down => [0, -1],
            Direction::Left => [-1, 0],
            Direction::Up => [0, 1],
        }
    }

    fn index_vec(self, vec: &Vec4) -> &f32 {
        match self {
            Direction::Right => &vec.x,
            Direction::Down => &vec.y,
            Direction::Left => &vec.z,
            Direction::Up => &vec.w,
        }
    }

    fn index_vec_mut(self, vec: &mut Vec4) -> &mut f32 {
        match self {
            Direction::Right => &mut vec.x,
            Direction::Down => &mut vec.y,
            Direction::Left => &mut vec.z,
            Direction::Up => &mut vec.w,
        }
    }

    fn map(mut f: impl FnMut(Direction) -> f32) -> Vec4 {
        let mut vec = Vec4::ZERO;
        for dir in Self::all() {
            *dir.index_vec_mut(&mut vec) = f(dir);
        }

        vec
    }

    fn map_offset(start: [i32; 2], mut f: impl FnMut([i32; 2], Direction) -> f32) -> Vec4 {
        Self::map(move |dir| {
            let offset = dir.offset();
            f([start[0] + offset[0], start[1] + offset[1]], dir)
        })
    }

    fn for_each(mut f: impl FnMut(Direction)) {
        Self::map(|dir| {
            f(dir);
            0.0
        });
    }

    fn for_each_offset(start: [i32; 2], mut f: impl FnMut([i32; 2], Direction)) {
        Self::map_offset(start, |coord, dir| {
            f(coord, dir);
            0.0
        });
    }

    fn single_component(self, value: f32) -> Vec4 {
        let mut vec = Vec4::ZERO;
        *self.index_vec_mut(&mut vec) = value;
        vec
    }

    fn swap(vec: Vec4) -> Vec4 {
        Vec4::new(vec.z, vec.w, vec.x, vec.y)
    }

    fn normalize(vec: Vec4) -> Vec4 {
        vec - Direction::swap(vec.min(Vec4::ZERO))
    }
}

impl GenericTiles<i32, LiquidTile> {
    pub fn step(&mut self, solid: &GenericTiles<i32, OptTileIndex>, t: f32) {
        let active_tiles = self.indexed_tiles()
            .filter(|(_, tile)| tile.amount >= 0.01)
            .map(|(coord, tile)| (coord, tile.clone()))
            .collect::<Vec<_>>();

        for (coord, tile) in active_tiles {
            if let Some(tile) = self.get_mut(&coord) {
                tile.velocity *= 0.9;
                //tile.velocity = Direction::normalize(tile.velocity);
            }

            let gradient = Direction::map_offset(coord, |offset_coord, _| {
                let amount = self.get_or_default(&offset_coord).amount;
                tile.amount - amount
            });

            let gravity =
                Direction::Down.single_component(1.0) + Direction::Up.single_component(-1.0);
            let force = (gradient + gravity * 0.1).max(Vec4::ZERO);

            let tile = self.get_or_create(&coord);
            let acceleration = force * t;
            tile.velocity += acceleration;

            Direction::for_each_offset(coord, |offset_coord, dir| {
                if solid.get(&offset_coord).and_then(|t| t.get_index()).is_some() {
                    *dir.index_vec_mut(&mut tile.velocity) = 0.0;
                }
            });

            let tile = tile.clone();

            //let flow_to_equilibrium = Direction::map_offset(coord)

            let total_velocity = tile.velocity.x + tile.velocity.y + tile.velocity.z + tile.velocity.w;
            let total_flow_rate = total_velocity.min(tile.amount);

            let mut total_outflow = 0.0;
            Direction::for_each_offset(coord, |offset_coord, dir| {
                let flow_rate = total_flow_rate * (*dir.index_vec(&tile.velocity) / total_velocity);
                let flow = flow_rate;
                if flow > 0.0 {
                    let target = self.get_or_create(&offset_coord);
                    target.amount += flow;
                    total_outflow += flow;
                    //let transfer_velocity = dir.index_vec(&tile.velocity) * (flow / flow_rate);
                    //*dir.index_vec_mut(&mut target.velocity) += transfer_velocity;
                    //*dir.index_vec_mut(&mut next.get_or_create(&coord).velocity) -= transfer_velocity;
                }
            });

            let tile = self.get_or_create(&coord);
            tile.amount -= total_outflow;
        }
    }

    fn get_max_inflow(
        &self,
        current_amount: f32,
        solid: &GenericTiles<i32, OptTileIndex>,
        point: [i32; 2],
    ) -> f32 {
        if solid.get(&point).and_then(|t| t.get_index()).is_some() {
            0.0
        } else {
            let amount = self.get(&point).map(|t| t.amount).unwrap_or(0.0);
            let max_inflow = (1.0 - amount).max((current_amount - amount) / 2.0).max(0.0);
            if max_inflow < 0.01 {
                0.0
            } else {
                max_inflow
            }
        }
    }
}

impl std::fmt::Display for LiquidTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let amount = (self.amount * 10.0).round() / 10.0;
        write!(f, "{}", amount)
    }
}
