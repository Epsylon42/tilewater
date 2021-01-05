use super::generic_tiles::*;
use super::OptTileIndex;

#[derive(Default, Clone)]
pub struct LiquidTile {
    pub amount: f32,
    flow: f32,
}

impl LiquidTile {
    pub fn new(amount: f32) -> Self {
        LiquidTile {
            amount,
            flow: 0.0,
        }
    }
}

impl GenericTiles<i32, LiquidTile> {
    pub fn step(&mut self, solid: &GenericTiles<i32, OptTileIndex>, t: f32) {
        eprintln!("flow");
        let mut next = self.clone();
        next.clear_modified();

        self.deduplicate_modified();
        for chunk_coord in self.modified_chunks() {
            let chunk = &self.chunks()[chunk_coord];
            for &inner_coord in chunk.modified_tiles() {
                let tile = &chunk.tiles()[inner_coord];

                let point = self.combine_coord(chunk_coord, &inner_coord);
                let point_below = [point[0], point[1] - 1];

                let max_outflow = tile.amount.min(t);
                let mut max_inflow = self
                    .get(&point_below)
                    .map(|t| 1.0 - t.amount)
                    .unwrap_or(0.0);
                if solid
                    .get(&point_below)
                    .and_then(|t| t.get_index())
                    .is_some()
                {
                    max_inflow = 0.0;
                }
                if max_outflow != 0.0 {
                    dbg!(max_outflow);
                }
                if max_inflow != 0.0 {
                    dbg!(max_inflow);
                }

                let flow = max_inflow.min(max_outflow);
                next.get_or_create(&point).amount -= flow;
                next.get_or_create(&point_below).amount += flow;
            }
        }

        *self = next;
    }
}

impl std::fmt::Display for LiquidTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let amount = (self.amount * 10.0).round() / 10.0;
        write!(f, "{}", amount)
    }
}
