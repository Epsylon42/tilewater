use ndarray::prelude::*;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct GenericChunk<T> {
    tiles: Array2<T>,
    modified: Vec<[usize; 2]>,
}

impl<T: Default> GenericChunk<T> {
    pub fn empty(size: usize) -> Self {
        GenericChunk {
            tiles: Array2::from_shape_fn([size; 2], |_| Default::default()),
            modified: Vec::new(),
        }
    }
}

impl<T> GenericChunk<T> {
    pub fn indexed_tiles(&self) -> impl Iterator<Item = ([usize; 2], &T)> {
        self.tiles
            .indexed_iter()
            .map(|((x, y), tile)| ([x, y], tile))
    }

    pub fn modified_tiles(&self) -> &[[usize; 2]] {
        &self.modified
    }

    pub fn clear_modified(&mut self) {
        self.modified.clear();
    }

    pub fn tiles(&self) -> &Array2<T> {
        &self.tiles
    }
}

#[derive(Default, Clone)]
pub struct GenericTiles<C, T> {
    chunk_size: usize,
    chunks: HashMap<[C; 2], GenericChunk<T>>,
    modified: Vec<[C; 2]>,
}

type C = i32;

impl<T: Default> GenericTiles<i32, T> {
    pub fn new(chunk_size: usize) -> Self {
        GenericTiles {
            chunk_size,
            chunks: HashMap::new(),
            modified: Vec::new(),
        }
    }

    pub fn indexed_chunks(&self) -> impl Iterator<Item = (&[C; 2], &GenericChunk<T>)> {
        self.chunks.iter()
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.modified.clear();
    }

    pub fn modified_chunks(&self) -> &[[C; 2]] {
        &self.modified
    }

    pub fn deduplicate_modified(&mut self) {
        self.modified.sort();
        self.modified.dedup();

        for chunk_coord in &self.modified {
            if let Some(chunk) = self.chunks.get_mut(chunk_coord) {
                chunk.modified.sort();
                chunk.modified.dedup();
            }
        }
    }

    pub fn clear_modified(&mut self) {
        for chunk_coord in &self.modified {
            self.chunks.get_mut(chunk_coord).map(|c| c.clear_modified());
        }
        self.modified.clear();
    }

    pub fn get_chunk_or_create(&mut self, coord: [C; 2]) -> &mut GenericChunk<T> {
        let chunk_size = self.chunk_size;
        self.modified.push(coord);
        self.chunks
            .entry(coord)
            .or_insert_with(|| GenericChunk::empty(chunk_size))
    }

    pub fn chunks(&self) -> &HashMap<[C; 2], GenericChunk<T>> {
        &self.chunks
    }

    pub fn set(&mut self, point: &[C; 2], tile: T) {
        let chunk_coord = self.point_to_chunk_coord(point);
        let inner_coord = self.point_to_inner_coord(point);
        let chunk = self.get_chunk_or_create(chunk_coord);
        chunk.tiles[inner_coord] = tile;
        chunk.modified.push(inner_coord);
        self.modified.push(chunk_coord);
    }

    pub fn get(&self, point: &[C; 2]) -> Option<&T> {
        let chunk_coord = self.point_to_chunk_coord(point);
        let inner_coord = self.point_to_inner_coord(point);
        self.chunks.get(&chunk_coord)?.tiles.get(inner_coord)
    }

    pub fn get_mut(&mut self, point: &[C; 2]) -> Option<&mut T> {
        let chunk_coord = self.point_to_chunk_coord(point);
        let inner_coord = self.point_to_inner_coord(point);
        let chunk = self.chunks.get_mut(&chunk_coord)?;
        chunk.modified.push(inner_coord);
        self.modified.push(chunk_coord);
        chunk.tiles.get_mut(inner_coord)
    }

    pub fn get_or_create(&mut self, point: &[C; 2]) -> &mut T {
        let chunk_coord = self.point_to_chunk_coord(point);
        let inner_coord = self.point_to_inner_coord(point);
        self.modified.push(chunk_coord);
        let chunk = self.get_chunk_or_create(chunk_coord);
        chunk.modified.push(inner_coord);
        &mut chunk.tiles[inner_coord]
    }

    pub fn point_to_chunk_coord(&self, point: &[C; 2]) -> [C; 2] {
        coord::point_to_chunk_coord(self.chunk_size, point)
    }

    pub fn point_to_inner_coord(&self, point: &[C; 2]) -> [usize; 2] {
        coord::point_to_inner_coord(self.chunk_size, point)
    }

    pub fn chunk_coord_to_corner(&self, chunk_coord: &[C; 2]) -> [C; 2] {
        coord::chunk_coord_to_corner(self.chunk_size, chunk_coord)
    }

    pub fn combine_coord(&self, chunk_coord: &[C; 2], inner_coord: &[usize; 2]) -> [C; 2] {
        coord::combine_coord(self.chunk_size, chunk_coord, inner_coord)
    }

    pub fn combine_coord_tuple(&self, chunk_coord: &[C; 2], (x, y): (usize, usize)) -> [C; 2] {
        coord::combine_coord(self.chunk_size, chunk_coord, &[x, y])
    }

    pub fn split_coord(&self, point: &[C; 2]) -> ([C; 2], [usize; 2]) {
        coord::split_coord(self.chunk_size, point)
    }
}

pub mod coord {
    pub type C = i32;

    pub fn point_to_chunk_coord(chunk_size: usize, point: &[C; 2]) -> [C; 2] {
        map(point, |c| c.div_euclid(chunk_size as C))
    }

    pub fn point_to_inner_coord(chunk_size: usize, point: &[C; 2]) -> [usize; 2] {
        map(point, |c| c.rem_euclid(chunk_size as C) as usize)
    }

    pub fn chunk_coord_to_corner(chunk_size: usize, chunk_coord: &[C; 2]) -> [C; 2] {
        map(chunk_coord, |c| *c * chunk_size as C)
    }

    pub fn combine_coord(
        chunk_size: usize,
        chunk_coord: &[C; 2],
        inner_coord: &[usize; 2],
    ) -> [C; 2] {
        zip_map(chunk_coord, inner_coord, |c, i| {
            c * chunk_size as C + *i as C
        })
    }

    pub fn split_coord(chunk_size: usize, point: &[C; 2]) -> ([C; 2], [usize; 2]) {
        let chunk_coord = point_to_chunk_coord(chunk_size, point);
        let inner_coord = point_to_inner_coord(chunk_size, point);
        (chunk_coord, inner_coord)
    }

    fn map<T, U>([x, y]: &[T; 2], f: impl Fn(&T) -> U) -> [U; 2] {
        [f(x), f(y)]
    }

    fn zip_map<T, U, R>([xa, ya]: &[T; 2], [xb, yb]: &[U; 2], f: impl Fn(&T, &U) -> R) -> [R; 2] {
        [f(xa, xb), f(ya, yb)]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn point_to_chunk_coord() {
        let tiles = GenericTiles::<i32, ()>::new(16);
        assert_eq!(tiles.point_to_chunk_coord(&[0, 0]), [0, 0]);
        assert_eq!(tiles.point_to_chunk_coord(&[10, 10]), [0, 0]);
        assert_eq!(tiles.point_to_chunk_coord(&[16, 16]), [1, 1]);
        assert_eq!(tiles.point_to_chunk_coord(&[-5, -5]), [-1, -1]);
        assert_eq!(tiles.point_to_chunk_coord(&[-15, -15]), [-1, -1]);
        assert_eq!(tiles.point_to_chunk_coord(&[-16, -16]), [-1, -1]);
        assert_eq!(tiles.point_to_chunk_coord(&[-17, -17]), [-2, -2]);
    }

    #[test]
    fn point_to_inner_coord() {
        let tiles = GenericTiles::<i32, ()>::new(16);
        assert_eq!(tiles.point_to_inner_coord(&[0, 0]), [0, 0]);
        assert_eq!(tiles.point_to_inner_coord(&[10, 10]), [10, 10]);
        assert_eq!(tiles.point_to_inner_coord(&[16, 16]), [0, 0]);
        assert_eq!(tiles.point_to_inner_coord(&[-1, -1]), [15, 15]);
        assert_eq!(tiles.point_to_inner_coord(&[-2, -2]), [14, 14]);
        assert_eq!(tiles.point_to_inner_coord(&[-16, -16]), [0, 0]);
        assert_eq!(tiles.point_to_inner_coord(&[-17, -17]), [15, 15]);
    }

    #[test]
    fn chunk_coord_to_corner() {
        let tiles = GenericTiles::<i32, ()>::new(16);
        assert_eq!(tiles.chunk_coord_to_corner(&[0, 0]), [0, 0]);
        assert_eq!(tiles.chunk_coord_to_corner(&[1, 1]), [16, 16]);
        assert_eq!(tiles.chunk_coord_to_corner(&[-1, -1]), [-16, -16]);
        assert_eq!(tiles.chunk_coord_to_corner(&[-2, -2]), [-32, -32]);
    }

    #[test]
    fn combine_coord() {
        let tiles = GenericTiles::<i32, ()>::new(16);

        let points = [
            [0, 0],
            [10, 10],
            [16, 16],
            [-5, -5],
            [-15, -15],
            [-16, -16],
            [-17, -17],
        ];

        for point in &points {
            let chunk_coord = tiles.point_to_chunk_coord(point);
            let inner_coord = tiles.point_to_inner_coord(point);
            assert_eq!(tiles.combine_coord(&chunk_coord, &inner_coord), *point);
        }
    }
}
