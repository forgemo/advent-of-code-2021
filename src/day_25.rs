use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use itertools::{Itertools};
use pathfinding::prelude::bfs;
use rayon::iter::plumbing::{Consumer, ProducerCallback, UnindexedConsumer};
use rayon::prelude::{ParallelIterator, IntoParallelIterator, IntoParallelRefIterator, IndexedParallelIterator};
use regex::Regex;
use crate::lib::read_lines;

mod lib;


fn main() {
    let task_a = task_a(read_lines("input/day_25.txt"));
    assert_eq!(580810, task_a);
    let task_b = task_b(read_lines("input/day_25.txt"));
    assert_eq!(1265621119006734, task_b);
    println!("task_a: {}, task_b: {}", task_a, task_b);
}


fn task_a(lines: impl Iterator<Item=String>) -> usize {
    let mut a = Map::from_lines(lines);
    a.move_until_stop()
}

fn task_b(lines: impl Iterator<Item=String>) -> isize {
    todo!()
}


struct Map {
    grid: Vec<Vec<char>>,
    width: usize,
    height: usize
}

impl Map {
    fn from_lines(lines: impl Iterator<Item=String>) -> Map {
        let grid = lines.map(|l|l.chars().collect_vec()).collect_vec();
        let width = grid[0].len();
        let height = grid.len();
        Map { grid, width, height}
    }

    fn visit(&self, c: &char, dir: (usize, usize)) -> Vec<((usize, usize),(usize, usize))> {
        self.grid.iter()
            .enumerate()
            .rev()
            .flat_map(|(y, r)|
                r.iter().enumerate().rev().map(move |(x, v)| (x, y, v)))
            .filter_map(|(x, y, v)| {
                if v == c {
                    let (to_x, to_y) = ((x+dir.0) % self.width, (y+dir.1) % self.height);
                    //println!("{:?}->{:?}", (x, y), (to_x, to_y));
                    let is_to_free = self.grid[to_y][to_x] == '.';
                    if is_to_free { Some(((x,y), (to_x, to_y)))  }
                    else { None }
                } else {
                    None
                }
            })
            .collect_vec()
    }

    fn move_all_down(&mut self) -> usize {
        let v = self.visit(  &'v', (0, 1));
        let count = v.len();
        for ((from_x, from_y), (to_x, to_y)) in v {
            self.grid[to_y][to_x] = 'v';
            self.grid[from_y][from_x] = '.';
        }
        count
    }

    fn move_all_right(&mut self) -> usize{
        let v = self.visit(  &'>', (1, 0));
        let count = v.len();
        for ((from_x, from_y), (to_x, to_y)) in v {
            self.grid[to_y][to_x] = '>';
            self.grid[from_y][from_x] = '.';
        }
        count
    }

    fn step(&mut self) -> usize {
        self.move_all_right() +
        self.move_all_down()
    }

    fn move_until_stop(&mut self) -> usize {
        let mut i = 0;
        while self.step() > 0 {
            //println!("{}\n\n", self);
            i+=1;
        }
        i+1
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = self.grid.iter()
            .map(|l| l.iter().join(""))
            .join("\n");
        write!(f, "{}", s)
    }
}


#[cfg(test)]
mod tests {


    #[test]
    fn test_0() {

    }

}

