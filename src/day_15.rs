mod lib;

use std::ops::Mul;
use itertools::{Itertools,};
use pathfinding::prelude::astar;
use crate::lib::{read_lines};

fn main() {
    let result_a = task(read_lines("input/day_15.txt"), false);
    assert_eq!(result_a, 472);

    let result_b = task(read_lines("input/day_15.txt"), true);
    assert_eq!(result_b, 2851);

    println!("task-a: {}, task-b: {}", result_a, result_b);
}

fn task(lines: impl Iterator<Item=String>, large_map: bool) -> usize {
    let mut grid = read_input(lines);
    if large_map { grid = grid * 5 };
    let start = (0usize, 0usize);
    let goal = (grid.width - 1, grid.height - 1);

    let (_, risk) = astar(&start,
                             |&(x, y)| neighbours(&grid, x, y).iter().map(|(x, y, r)| ((*x, *y), *r)).collect_vec(),
                             |&(x, y)| (((x as isize - goal.0 as isize).abs() + (y as isize - goal.1 as isize).abs()) / 3) as usize,
                             |&p| p == goal).unwrap();

    risk
}

fn neighbours(grid: &Grid, x: usize, y: usize) -> Vec<(usize, usize, usize)> {
    vec![(x as isize - 1, y as isize), ((x + 1) as isize, y as isize),
         (x as isize, y as isize - 1), (x as isize, (y + 1) as isize)]
        .into_iter().filter_map(|(x, y)| {
        if x < 0 || x >= grid.width as isize || y < 0 || y >= grid.height as isize
        { None } else { Some((x as usize, y as usize)) }
    }).map(|(x, y)| (x, y, grid.fields[x][y]))
        .collect()
}

struct Grid {
    fields: Vec<Vec<usize>>,
    width: usize,
    height: usize,
}

impl Mul<usize> for Grid {
    type Output = Grid;

    fn mul(self, rhs: usize) -> Self::Output {
        let mut fields = vec![vec![0usize; self.height * rhs]; self.width * rhs];
        let (width, height) = (fields.len(), fields[0].len());

        for x in 0..self.width {
            for y in 0..self.height {
                fields[x][y] = self.fields[x][y];
            }
        }

        for ix in 0..rhs{
            for iy in 0..rhs {
                if ix == 0 && iy == 0 {
                    continue;
                }
                for x in 0..self.width {
                    for y in 0..self.height {

                        let target_project = (ix*self.width, iy*self.height);
                        let target_x = x + target_project.0;
                        let target_y = y + target_project.1;

                        let source_project = if ix == 0 {
                            (0, self.height)
                        } else {
                            (self.width, 0)
                        };
                        let source_x = target_x-source_project.0;
                        let source_y = target_y-source_project.1;

                        fields[target_x][target_y] = fields[source_x][source_y] +1;
                        if fields[target_x][target_y] > 9 {
                            fields[target_x][target_y] = 1
                        }
                    }
                }
            }
        }

        Grid { fields, height, width }
    }
}

fn read_input(lines: impl Iterator<Item=String>) -> Grid {
    let fields = lines.map(|line| line.chars()
        .map(|c| c.to_digit(10).expect("failed to parse") as usize).collect_vec()
    ).collect_vec();
    let (width, height) = (fields.len(), fields[0].len());
    Grid { fields, height, width }
}