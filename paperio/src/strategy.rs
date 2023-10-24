use crate::data::{Cell, Direction, Point, World, MAP_SIZE_CELLS};
use std::cmp::max;

pub struct Strategy {
    prev_direction: Direction,
    best_direction: Direction,
    perimeter_cells: Vec<Point>,
}

impl Default for Strategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy {
    pub fn new() -> Self {
        Strategy {
            prev_direction: Direction::Down,
            best_direction: Direction::Down,
            perimeter_cells: Vec::new(),
        }
    }

    pub fn on_tick(&mut self, world: World) -> Direction {
        let my_position = world.me().position;
        let cur_cells = world.me().territory.clone();
        let neighbour_left = Cell(my_position.to_cell().0 - 1, my_position.to_cell().1);
        let neighbour_down = Cell(my_position.to_cell().0, my_position.to_cell().1 - 1);
        let neighbour_right = Cell(my_position.to_cell().0 + 1, my_position.to_cell().1);
        let neighbour_up = Cell(my_position.to_cell().0, my_position.to_cell().1 + 1);
        if cur_cells.contains(&my_position) {
            let mut best_rectangle_score = i32::MIN;
            for x1 in 0..=my_position.to_cell().0 {
                for y1 in 0..=my_position.to_cell().1 {
                    for x2 in x1..=MAP_SIZE_CELLS / 2 {
                        for y2 in y1..=MAP_SIZE_CELLS / 2 {
                            let bottom_left = Point(30 * x1, 30 * y1);
                            let top_right = Point(30 * x2, 30 * y2);
                            let bottom_left_cell = bottom_left.to_cell();
                            let top_right_cell = top_right.to_cell();
                            if bottom_left_cell.in_bounds() && top_right_cell.in_bounds() {
                                let cells_score =
                                    calculate_cells_score(&world, &my_position, &bottom_left);
                                let cells_score1 =
                                    calculate_cells_score(&world, &top_right, &my_position);
                                let danger = calculate_danger(&world, &top_right, &bottom_left);
                                let danger1 = calculate_danger(&world, &top_right, &bottom_left);
                                let a = cells_score - danger * danger;
                                let a1 = cells_score1 - danger1 * danger1;
                                let rectangle_score = max(a, a1);
                                if rectangle_score > best_rectangle_score {
                                    best_rectangle_score = rectangle_score;
                                    if a >= a1 {
                                        self.perimeter_cells =
                                            calculate_perimeter_cells(my_position, bottom_left);
                                    } else {
                                        self.perimeter_cells =
                                            calculate_perimeter_cells(top_right, my_position);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            self.directions(
                &neighbour_left,
                &neighbour_down,
                &neighbour_right,
                &neighbour_up,
            );
            self.prev_direction = self.best_direction;

            self.best_direction
        } else {
            self.directions(
                &neighbour_left,
                &neighbour_down,
                &neighbour_right,
                &neighbour_up,
            );
            self.prev_direction = self.best_direction;
            self.prev_direction
        }
    }

    fn directions(
        &mut self,
        neighbour_left: &Cell,
        neighbour_down: &Cell,
        neighbour_right: &Cell,
        neighbour_up: &Cell,
    ) {
        if neighbour_left.in_bounds()
            && self
                .perimeter_cells
                .contains(&Point(30 * neighbour_left.0, 30 * neighbour_left.1))
            && self.prev_direction != Direction::Right
        {
            self.best_direction = Direction::Left;
        } else if neighbour_down.in_bounds()
            && self
                .perimeter_cells
                .contains(&Point(30 * neighbour_down.0, 30 * neighbour_down.1))
            && self.prev_direction != Direction::Up
        {
            self.best_direction = Direction::Down;
        } else if neighbour_right.in_bounds()
            && self
                .perimeter_cells
                .contains(&Point(30 * neighbour_right.0, 30 * neighbour_right.1))
            && self.prev_direction != Direction::Left
        {
            self.best_direction = Direction::Right;
        } else if neighbour_up.in_bounds()
            && self
                .perimeter_cells
                .contains(&Point(30 * neighbour_up.0, 30 * neighbour_up.1))
            && self.prev_direction != Direction::Down
        {
            self.best_direction = Direction::Up;
        }
    }
}

fn calculate_danger(world: &World, top_right: &Point, bottom_left: &Point) -> i32 {
    let perimiter = ((top_right.to_cell().0 - bottom_left.to_cell().0)
        + (top_right.to_cell().1 - bottom_left.to_cell().1))
        * 2;
    let my_territory = &world.me().territory;
    let mut min_enemy_distance = i32::MAX;
    for x in bottom_left.to_cell().0..=top_right.to_cell().0 {
        for y in bottom_left.to_cell().1..=top_right.to_cell().1 {
            let cell_position = Point(30 * x, 30 * y);
            if cell_position.to_cell().in_bounds() && !my_territory.contains(&cell_position) {
                let enemy_distance = calculate_enemy_min_distance(world, &cell_position);
                if enemy_distance < min_enemy_distance {
                    min_enemy_distance = enemy_distance;
                }
            }
        }
    }
    perimiter - min_enemy_distance
}

fn calculate_enemy_min_distance(world: &World, cell_position: &Point) -> i32 {
    let mut min_distance = i32::MAX;

    for (_, enemy) in world.iter_enemies() {
        for enemy_cell in &enemy.territory {
            let distance = cell_position.to_cell().distance_to(enemy_cell.to_cell());
            if distance < min_distance {
                min_distance = distance;
            }
        }
    }

    min_distance
}

fn calculate_cells_score(world: &World, top_right: &Point, bottom_left: &Point) -> i32 {
    let mut cells_score = 0;
    let cells = world.me().territory.clone();

    for x in bottom_left.to_cell().0..=top_right.to_cell().0 {
        for y in bottom_left.to_cell().1..=top_right.to_cell().1 {
            let cell_position = Point(30 * x, 30 * y);
            if cell_position.to_cell().in_bounds() {
                if !cells.contains(&cell_position) && is_enemy_territory(world, &cell_position) {
                    cells_score += 5;
                } else if !cells.contains(&cell_position)
                    && !is_enemy_territory(world, &cell_position)
                {
                    cells_score += 1;
                }
            }
        }
    }
    cells_score
}

fn is_enemy_territory(world: &World, cell_position: &Point) -> bool {
    for player in world.iter_enemies() {
        let enemy_cells = player.1.territory.clone();
        if enemy_cells.contains(cell_position) {
            return true;
        }
    }
    false
}

fn calculate_perimeter_cells(top_right: Point, bottom_left: Point) -> Vec<Point> {
    let mut perimeter = Vec::new();
    for x in bottom_left.to_cell().0..=top_right.to_cell().0 {
        perimeter.push(Point(30 * x, 30 * top_right.to_cell().1));
        perimeter.push(Point(30 * x, 30 * bottom_left.to_cell().1));
    }

    for y in bottom_left.to_cell().1 + 1..top_right.to_cell().1 {
        perimeter.push(Point(30 * bottom_left.to_cell().0, 30 * y));
        perimeter.push(Point(30 * top_right.to_cell().0, y * 30));
    }

    perimeter
}
