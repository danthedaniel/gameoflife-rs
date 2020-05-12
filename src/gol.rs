use std::ops::{Index, IndexMut};

use glium::texture::RawImage2d;
use rand::{thread_rng, RngCore};

/// Conway's Game of Life.
#[derive(Clone)]
pub struct GoL {
    /// Linear vector of all cells on the board. Cells are 0 when dead. Non-zero
    /// indicates how many generations a cell has been alive.
    pub buffer: Vec<u8>,
    pub width: i32,
    pub height: i32,
}

impl GoL {
    pub fn new(dims: (usize, usize)) -> GoL {
        GoL {
            buffer: vec![0; dims.0 * dims.1],
            width: dims.0 as i32,
            height: dims.1 as i32,
        }
    }

    /// Reset and randomize all cells.
    pub fn randomize(&mut self) {
        let mut rng = thread_rng();

        for index in 0..self.buffer.len() {
            self.buffer[index] = (rng.next_u32() & 0x00000001) as u8;
        }
    }

    /// Execute one generation of the game.
    pub fn step(&mut self) {
        // Space... the final frontier.
        let mut next_gen = self.clone();

        for x in 0..self.width {
            for y in 0..self.height {
                next_gen[(x, y)] = self.automata_rules(x, y);
            }
        }

        self.buffer = next_gen.buffer;
    }

    /// Execute one generation on a single cell.
    #[inline]
    fn automata_rules(&self, x: i32, y: i32) -> u8 {
        let current_cell = self[(x, y)];
        let n_neighbors = self.alive_neighbors(x, y);

        match (n_neighbors, current_cell != 0) {
            (0..=1, true) => 0,                              // Underpopulated
            (2..=3, true) => current_cell.saturating_add(1), // Goldilocks zone
            (3..=8, true) => 0,                              // Overcrowded
            (3, false) => current_cell.saturating_add(1),    // Spontaneous reproduction
            _ => 0,                                          // From nothing comes nothing
        }
    }

    /// Count living cells adjacent to a cell in the matrix.
    #[inline]
    #[rustfmt::skip]
    fn alive_neighbors(&self, x: i32, y: i32) -> u8 {
        [
            self[(x - 1, y - 1)], self[(x + 0, y - 1)], self[(x + 1, y - 1)],
            self[(x - 1, y + 0)], /*  selected cell  */ self[(x + 1, y + 0)],
            self[(x - 1, y + 1)], self[(x + 0, y + 1)], self[(x + 1, y + 1)],
        ]
        .iter()
        .fold(0, |total, &neighbor| total + (neighbor != 0) as u8)
    }

    /// Convert to an image for use by Glium.
    pub fn as_raw_image_2d(&self) -> RawImage2d<'static, u8> {
        // TODO: This needs to be built left-to-right, bottom-to-top. Currently
        // it's top-to-bottom so the texture is flipped.
        let mut image_data = vec![0u8; (self.width * self.height * 4) as usize];

        for (index, &cell) in self.buffer.iter().enumerate() {
            let val = if cell > 0 { 255 } else { 0 };
            image_data[4 * index + 0] = val;
            image_data[4 * index + 1] = val;
            image_data[4 * index + 2] = val;
            image_data[4 * index + 3] = cell;
        }

        RawImage2d::from_raw_rgba(image_data, (self.width as u32, self.height as u32))
    }
}

impl Index<(i32, i32)> for GoL {
    type Output = u8;

    fn index(&self, index: (i32, i32)) -> &u8 {
        let x = index.0 % self.width;
        let y = index.1 % self.height;

        &self
            .buffer
            .get((y * self.height + x) as usize)
            .unwrap_or(&0)
    }
}

impl IndexMut<(i32, i32)> for GoL {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut u8 {
        let x = index.0 % self.width;
        let y = index.1 % self.height;

        &mut self.buffer[(y * self.height as i32 + x) as usize]
    }
}
