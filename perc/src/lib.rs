#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

use rand::random;

/// Represents a grid of boolean values.
pub struct BoolGrid {
    // TODO: your code here.
    vector_2d: Vec<Vec<bool>>,
}

impl BoolGrid {
    /// * `width` - grid width.
    /// * `height` - grid height.
    pub fn new(width: usize, height: usize) -> Self {
        // TODO: your code here.
        let vector_2d = vec![vec![false; width]; height];
        Self { vector_2d }
    }

    /// Creates a new grid with every value initialized randomly.
    /// * `vacancy` - probability of any given value being equal
    /// to `false`.
    pub fn random(width: usize, height: usize, vacancy: f64) -> Self {
        // TODO: your code here.
        let mut vector_2d = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row_vector = Vec::with_capacity(width);
            for _ in 0..width {
                let manh = random::<f64>() >= vacancy;
                row_vector.push(manh);
            }
            vector_2d.push(row_vector);
        }
        Self { vector_2d }
    }

    /// Returns grid width.
    pub fn width(&self) -> usize {
        // TODO: your code here.
        return self.vector_2d[0].len();
    }

    /// Returns grid height.
    pub fn height(&self) -> usize {
        // TODO: your code here.
        return self.vector_2d.len();
    }

    /// Returns the current value of a given cell.
    /// If `x` or `y` is out of bounds, this method may panic
    /// (or return incorrect result).
    pub fn get(&self, x: usize, y: usize) -> bool {
        // TODO: your code here.
        self.vector_2d[y][x]
    }

    /// Sets a new value to a given cell.
    /// If `x` or `y` is out of bounds, this method may panic
    /// (or set value to some other unspecified cell).
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        // TODO: your code here.
        self.vector_2d[y][x] = value;
    }

    // TODO: your code here.
}

////////////////////////////////////////////////////////////////////////////////
/// from any cell with `y` == 0 to any cell with `y` == `height` - 1.
/// If the grid is empty (`width` == 0 or `height` == 0), it percolates.
pub fn percolates(grid: &BoolGrid) -> bool {
    // TODO: your code here.
    if grid.height() == 0 || grid.width() == 0 {
        return true;
    }

    let mut visited = vec![vec![false; grid.width()]; grid.height()];

    for x in 0..grid.width() {
        if dfs(x, 0, &grid, &mut visited) {
            return true;
        }
    }
    false
}

fn dfs(x: usize, y: usize, grid: &BoolGrid, mut visited: &mut Vec<Vec<bool>>) -> bool {
    if x >= grid.width() || y >= grid.height() || grid.get(x, y) || visited[y][x] {
        return false;
    }

    if y == grid.height() - 1 {
        return true;
    }

    visited[y][x] = true;

    let next_to: [(isize, isize); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];

    for i in 0..next_to.len() {
        let next_x = x as isize + next_to[i].0;
        let next_y = y as isize + next_to[i].1;

        if next_x >= 0
            && next_y >= 0
            && next_x < grid.width() as isize
            && next_y < grid.height() as isize
        {
            if dfs(next_x as usize, next_y as usize, &grid, &mut visited) {
                return true;
            }
        }
    }
    return false;
}

////////////////////////////////////////////////////////////////////////////////

const N_TRIALS: u64 = 10000;

/// Returns an estimate of the probability that a random grid with given
/// `width, `height` and `vacancy` probability percolates.
/// To compute an estimate, it runs `N_TRIALS` of random experiments,
/// in each creating a random grid and checking if it percolates.
pub fn evaluate_probability(width: usize, height: usize, vacancy: f64) -> f64 {
    let mut perc_count = 0;
    for _ in 0..N_TRIALS {
        let grid = BoolGrid::random(width, height, vacancy);
        if percolates(&grid) {
            perc_count += 1;
        }
    }
    return perc_count as f64 / N_TRIALS as f64;
}
