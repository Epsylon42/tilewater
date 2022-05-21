use super::generic_tiles::*;
use super::OptTileIndex;

#[derive(Default, Clone)]
pub struct LiquidTile {
    pub amount: f32,
}

impl LiquidTile {
    pub fn new(amount: f32) -> Self {
        LiquidTile { amount }
    }

    pub fn is_empty(&self) -> bool {
        self.amount == 0.0
    }
}

impl GenericTiles<i32, LiquidTile> {
    pub fn step(&mut self, solid: &GenericTiles<i32, OptTileIndex>, _t: f32) {
        let mut next = self.clone();
        next.clear_modified();

        self.deduplicate_modified();
        for (chunk_coord, chunk) in self.indexed_chunks() {
            for (inner_coord, tile) in chunk.indexed_tiles() {
                let point = self.combine_coord(chunk_coord, &inner_coord);
                let point_below = [point[0], point[1] - 1];

                let amount = tile.amount;
                if amount == 0.0 {
                    continue;
                }


                let max_inflow_below = self.get_max_inflow(amount, solid, point_below);
                let inflow_below = max_inflow_below.min(amount);

                if inflow_below != 0.0 {
                    next.get_or_create(&point).amount -= inflow_below;
                    next.get_or_create(&point_below).amount += inflow_below;
                }

                if amount > inflow_below && amount > 0.1 {
                    let amount = amount - inflow_below;
                    let point_left = [point[0] - 1, point[1]];
                    let point_right = [point[0] + 1, point[1]];

                    let max_inflow_left = self.get_max_inflow(amount, solid, point_left);
                    let max_inflow_right = self.get_max_inflow(amount, solid, point_right);
                    let amount_lcr = max_inflow_left + max_inflow_right + amount;

                    if max_inflow_left + max_inflow_right != 0.0 {
                        let flow_left = amount * max_inflow_left / amount_lcr;
                        let flow_right = amount * max_inflow_right / amount_lcr;

                        next.get_or_create(&point).amount -= flow_left + flow_right;
                        next.get_or_create(&point_left).amount += flow_left;
                        next.get_or_create(&point_right).amount += flow_right;
                    }
                }

                if next.get(&point).unwrap().amount <= 0.01 {
                    next.get_or_create(&point).amount = 0.0;
                }
            }
        }

        *self = next;
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
