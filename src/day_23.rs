use std::array::from_ref;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut};
use std::ptr;
use std::ptr::hash;
use std::rc::Rc;
use std::slice::SliceIndex;
use itertools::{Itertools};
use pathfinding::prelude::{astar, bfs};
use rayon::iter::Empty;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use crate::Cell::*;
use crate::CellType::{Doorstep, Floor, Room};
use crate::lib::read_lines;
use crate::Occupant::{Amphipod, Wall};

mod lib;

fn main() {
    let task_a = task_a(read_lines("input/day_23.txt"));
    println!("result-a: {}", task_a);
}

fn task_a(lines: impl Iterator<Item=String>) -> usize {
    let energy_map = HashMap::from([
        ('A', 1),('B', 10),('C', 100),('D', 1000),
    ]);
    let start = Map::from(lines, energy_map);

    let result = astar(&start, |m|
        m.successors(),
          |m| m.heuristic(),
          |m| m.is_goal());
    println!("{:?}", result);
    result.unwrap().1
}


type AmphiColor = char;

type RoomId = char;
type WrongColorCount = RefCell<usize>;
type MustEnterRoom = bool;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum Occupant {
    Amphipod(AmphiColor, MustEnterRoom),
    Wall,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum CellType {
    Floor,
    Doorstep(RoomId),
    Room(RoomId),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum Cell {
    Empty(CellType),
    Occupied(CellType, Occupant),
}

impl Default for Cell {
    fn default() -> Self {
        Self::Empty(CellType::Floor)
    }
}


// --------------- Map --------------------

type Steps = usize;
type EnergyUse = usize;

#[derive(Debug, Eq, Clone)]
struct Map {
    grid: Grid<Cell>,
    wcc_map: HashMap<RoomId, WrongColorCount>,
    energy_map: HashMap<AmphiColor, EnergyUse>,
    room_cell_count: usize,
}

impl Hash for Map {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.grid.hash(h)
    }
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.grid.eq(&other.grid)
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = self.grid.rows.iter().map(|row| row.iter().map(|c| match c {
            Empty(_) => ' ',
            Occupied(Floor, Wall) => '▒',
            Occupied(_, Amphipod(id, _)) => *id,
            _ => panic!("unexpected {:?}", c)
        }).join("")).join("\n");

        write!(f, "{}", s)
    }
}

impl Map {

    pub fn from(
        lines: impl Iterator<Item=String>,
        energy_map: HashMap<AmphiColor, EnergyUse>) -> Self {

        let mut wcc_map = HashMap::new();
        wcc_map.extend(energy_map.keys()
            .map(|room|(*room, RefCell::new(0))));


        let rows = lines
            .map(|l| l.chars().collect_vec())
            .map(|chars| {
                let mut color_stack = energy_map.keys().sorted().collect_vec();
                color_stack.reverse();
                chars.into_iter()
                    .map(|c| match c {
                    '.' => Empty(Floor),
                    ' '|'#' => Occupied(Floor, Wall),
                    amphi_color => {
                        let room_id = color_stack.pop().unwrap();
                        if *room_id != amphi_color {
                            wcc_map.get(room_id).unwrap().replace_with(|c| *c+1);
                        }
                        Occupied(
                            Room(*room_id),
                            Amphipod(amphi_color, false)
                        )
                    }
                }).collect_vec()
            }).collect_vec();

        let room_cell_count = energy_map.len() * 2;

        Self {
            grid: Grid::from(rows),
            wcc_map,
            energy_map,
            room_cell_count
        }
    }

    pub fn visit_reachable_from(&self, from: &Position) -> Vec<(&Cell, Position, Steps)> {
        self.recurse_reachable_from(from, 1, &mut HashSet::new()).into_iter()
            .filter(|to_cell| self.is_valid_move(&self[from], to_cell.0))
            .collect_vec()
    }

    pub fn visit_amphipods(&self) -> impl Iterator<Item=(&Cell, Position)> {
        self.grid.visit_cells_with_position().filter(|(c, _)| matches!(c, Occupied(_, Amphipod(_, _))))
    }

    fn recurse_reachable_from(&self, p: &Position, steps: Steps, visited: &mut HashSet<Position>) -> Vec<(&Cell, Position, Steps)> {
        let neighbours = p.neighbours().into_iter()
            .filter(|n| matches!(self[n], Cell::Empty(_)))
            .filter(|n| !visited.contains(n))
            .collect_vec();

        visited.extend(neighbours.iter().cloned());

        let mut next_neighbours = neighbours.iter()
            .flat_map(|n| self.recurse_reachable_from(n, steps + 1, visited))
            .collect_vec();

        next_neighbours.extend(neighbours.into_iter()
            .map(|n| (&self[&n], n, steps)));

        next_neighbours
    }

    fn is_valid_move(&self, from: &Cell, to: &Cell) -> bool {
        match (from, to) {
            (_, Empty(Doorstep(_))) => false,
            (Occupied(Floor, Amphipod(_, must_enter_room)), Empty(Floor)) if *must_enter_room => false,
            (Occupied(Room(a), _), Empty(Room(b))) if a == b => true,
            (Occupied(Room(_), Amphipod(a, _)), Empty(Room(b))) => {
                let deb = format!("{}", self);
                let vip = deb.contains("▒A▒D▒C▒A▒" )&& deb.contains("▒B▒C▒ ▒D▒");
                if vip && *a == 'C' && *b == 'C' {
                    println!("{:?} -> {:?}", from, to);
                    println!("{}s\n{}", self,  a == b && *self.wcc_map.get(b).unwrap().borrow() == 0);
                    println!("{:?}", self.wcc_map);
                }
                a == b && *self.wcc_map.get(b).unwrap().borrow() == 0
            },
            (Occupied(_, Amphipod(a, _)), Empty(Room(b))) => {
                a == b && *self.wcc_map.get(b).unwrap().borrow() == 0
            },
            (Occupied(Room(_), Amphipod(_, _)), Empty(Floor)) => true,
            _ => panic!("unexpected case {:?} -> {:?}", from, to)
        }
    }

    fn leave_field(&mut self, p: &Position) -> Cell {
        let replacement = match &self[p] {
            Occupied(Room(room), Amphipod(_, _)) => Empty(Room(*room)),
            Occupied(Floor, _) => Empty(Floor),
            _ => panic!("unexpected case {:?}", self[p])
        };
        self.grid.replace(p, replacement)
    }

    fn occupy_field(&mut self, p: &Position, amphi_color: AmphiColor) -> Cell {
        let replacement = match &self[p] {
            Empty(Room(id)) => Occupied(Room(*id), Amphipod(amphi_color, false)),
            Empty(Floor) => Occupied(Floor, Amphipod(amphi_color, true)),
            _ => panic!("unexpected case {:?}", self[p])
        };
        self.grid.replace(p, replacement)
    }

    fn move_amphipod(&mut self, from: &Position, to: &Position) {
        let from_cell = self.leave_field(from);
        let amphi_color = match &from_cell {
            Occupied(_, Amphipod(c, _)) => c,
            _ => panic!("unexpected case")
        };
        let to_cell = self.occupy_field(to, *amphi_color);
        self.update_wcc(&from_cell, &to_cell);
    }

    fn update_wcc(&mut self, from: &Cell, to: &Cell, ) {

        let update = match (from, to) {
            (Occupied(Room(from_room), Amphipod(amphi_color, _)), Empty(Room(to_room))) => {
                if to_room != from_room && from_room != amphi_color { Some(from_room) } else {None}
            },
            (Occupied(Room(from_room), Amphipod(amphi_color, _)), Empty(Floor)) => {
                if from_room != amphi_color { Some(from_room) } else {None}
            },
            (Occupied(_, Amphipod(_,_)), Empty(_)) => None,
            _ => panic!("unexpected {:?} {:?}", from, to)
        };
        if let Some(from_room) = update {
            self.wcc_map.get(from_room).unwrap().replace_with(|c|*c-1);
        }

    }

    fn successors(&self) -> Vec<(Self, EnergyUse)> {
        self.visit_amphipods()
            .map(|(cell, position)| match cell {
                Occupied(_, Amphipod(amphi_color, _)) => (amphi_color, position),
                _ => panic!("unexpected case {:?}", cell)
            })
            .flat_map(|(amphi_color, from)| {
                //println!("\n--from {:?}----\n{}\n", from, self);
                self.visit_reachable_from(&from).iter()
                    .map(|(_, to, steps)| {
                        let mut successor = self.clone();
                        successor.move_amphipod(&from, to);

                        let energy_use = self.energy_map.get(amphi_color).unwrap();
                        (successor, *steps * energy_use)
                    }).collect_vec()
            })
            .collect_vec()
    }

    fn heuristic(&self) -> usize {
        //todo: optimize by adding correct_cell_count to Self
        let amphipods_in_correct_cell_count = self.visit_amphipods().filter(|(c, p)|
            matches!(c, Occupied(Room(r), Amphipod(a, _)) if a == r)
        ).count();

        self.room_cell_count - amphipods_in_correct_cell_count
    }

    fn is_goal(&self) -> bool {
        self.heuristic() == 0
    }
}

impl Index<&Position> for Map {
    type Output = Cell;

    fn index(&self, index: &Position) -> &Self::Output {
        &self.grid[index]
    }
}


// --------------- GRID --------------------

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct Position(usize, usize);

impl Position {
    fn neighbours(&self) -> Vec<Self> {
        vec![
            Position(self.0 + 1, self.1),
            Position(self.0 - 1, self.1),
            Position(self.0, self.1 + 1),
            Position(self.0, self.1 - 1),
        ]
    }
}

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct Grid<T> where T: Default + Clone {
    rows: Vec<Vec<T>>,
}

impl<T> Grid<T> where T: Default + Clone {
    fn new(width: usize, height: usize) -> Self {
        Grid {
            rows: vec![vec![T::default(); width]; height]
        }
    }

    fn swap(&mut self, a: &Position, b: &Position) {
        unsafe {
            let pa: *mut T = &mut self[a];
            let pb: *mut T = &mut self[b];
            ptr::swap(pa, pb);
        }
    }

    fn replace(&mut self, at: &Position, mut with: T) -> T {
        unsafe {
            let pa: *mut T = &mut self[at];
            let pb: *mut T = &mut with;
            ptr::swap(pa, pb);
        }
        with
    }

    fn visit_rows(&self) -> impl Iterator<Item=&Vec<T>> {
        self.rows.iter()
    }

    fn visit_cells(&self) -> impl Iterator<Item=&T> {
        self.visit_rows().flat_map(|r| r.iter())
    }

    fn visit_cells_with_position(&self) -> impl Iterator<Item=(&T, Position)> {
        self.visit_rows()
            .enumerate()
            .flat_map(|(row_index, row)| row.iter()
                .enumerate()
                .map(move |(col_index, cell)| {
                    (cell, Position(col_index, row_index))
                }))
    }
}

impl<'a, T> Index<&'a Position> for Grid<T>
    where T: Default + Clone {
    type Output = T;

    fn index(&self, index: &'a Position) -> &Self::Output {
        &self.rows[index.1][index.0]
    }
}

impl <T>From<Vec<Vec<T>>> for Grid<T> where T: Default + Clone {
    fn from(rows: Vec<Vec<T>>) -> Self {
        Grid { rows }
    }
}

impl<'a, T> IndexMut<&'a Position> for Grid<T>
    where T: Default + Clone {
    fn index_mut(&mut self, index: &'a Position) -> &mut Self::Output {
        &mut self.rows[index.1][index.0]
    }
}


