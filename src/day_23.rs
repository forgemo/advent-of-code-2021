use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use itertools::{Itertools};
use pathfinding::prelude::bfs;
use regex::Regex;
use crate::lib::read_lines;

mod lib;

fn main() {
    let task_a = task_a(read_lines("input/day_23.txt"));
    assert_eq!(580810, task_a);
    let task_b = task_b(read_lines("input/day_23.txt"));
    assert_eq!(1265621119006734, task_b);
    println!("task_a: {}, task_b: {}", task_a, task_b);
}

fn task_a(lines: impl Iterator<Item=String>) -> isize {

    let tiles = "##############...........####B#C#B#D###  #A#D#C#A#    #########  ";
    let map = Map::from_tiles(&tiles.chars().collect_vec(), 13);

    let result = solve(&map, &mut HashMap::new());

    println!("{:?}", result);

    todo!()
}

fn task_b(lines: impl Iterator<Item=String>) -> isize {
    todo!()
}


fn parse_input(lines: impl Iterator<Item=String>) -> usize {
    todo!()
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
struct Amphipod {
    label: char,
    energy_per_step: usize,
    must_move_next: bool,
    color: ColorId
}

impl Amphipod {
    fn new(label: char, color: ColorId, energy_per_step: usize) -> Self {
        Self {
            label,
            energy_per_step,
            must_move_next: false,
            color
        }
    }

    fn from_char(color: &char) -> Option<Self> {
        let energy_per_step = match color {
            'A' => Some(1),
            'B' => Some(10),
            'C' => Some(100),
            'D' => Some(1000),
            _ => None
        };
        energy_per_step
            .map(|e|Self::new(*color, *color, e))
    }
}

type ColorId = char;
#[derive(Clone, PartialOrd, PartialEq, Eq, Hash)]
enum Entrance {None, ToMixedRoom, ToColoredRoom(ColorId), ToEmptyRoom}
impl Default for Entrance { fn default() -> Self { Self::None } }

#[derive(Clone, PartialOrd, PartialEq, Eq, Hash)]
enum Tile {Wall, Floor}

impl From<&char> for Tile {
    fn from(c: &char) -> Self {
        match c {
            '#' | ' ' => Self::Wall,
            _ => Self::Floor
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Wall => write!(f, "░"),
            Tile::Floor => write!(f, " ")
        }
    }
}

#[derive(Clone, PartialOrd, PartialEq, Eq, Hash)]
struct Map {
    burrow: Vec<Tile>,
    entrances: Vec<Entrance>,
    amphipods: Vec<Option<Amphipod>>,
    width: usize,
    height: usize
}

#[derive(Debug)]
struct Coords {
    x: isize,
    y: isize
}


impl Map {
    fn from_tiles(
        tiles: &[char],
        width: usize
    ) -> Self {
        let burrow: Vec<Tile> = tiles.iter().map(Tile::from).collect();
        let entrances = vec![Entrance::default(); burrow.len()];
        let amphipods = vec![None; burrow.len()];
        let height = burrow.len() / width;
        let mut map = Self {burrow, entrances, amphipods, width, height };

        tiles.iter().enumerate()
            .filter_map(|(i, a)| Amphipod::from_char(a).map(|a|(i,a)))
            .for_each(|(i,a)| {
                map.amphipods[i] = Some(a);
                let Coords{x, y: _} = map._index_to_coords(&i);
                map.set_entrance(&Coords{x, y: 1}, Entrance::ToMixedRoom)
        });

        map
    }

    fn _coords_to_index(&self, c: &Coords ) -> usize {
        c.y as usize * self.width + c.x as usize
    }

    fn _index_to_coords(&self, i: &usize) -> Coords {
        Coords {
            x: (i % self.width) as isize,
            y: (i / self.width) as isize
        }
    }

    fn set_entrance(&mut self, c: &Coords, e: Entrance) {
        let index = self._coords_to_index(c);
        self.entrances[index] = e;
    }

    fn set_amphipod(&mut self, c: &Coords, a: Option<Amphipod>) {
        let index = self._coords_to_index(c);
        self.amphipods[index] = a;
    }

    fn possible_moves_for(&self, amphipod_index: &usize) ->  Vec<Coords> {
        let from = self._index_to_coords(amphipod_index);
        let amphipod = self.amphipods[*amphipod_index].as_ref().unwrap();
        self.free_spaces_around(&from).into_iter()
            .filter(|to|{
                let entering_room = from.y == 1 && to.y == 2;
                !entering_room || self.entrances[*amphipod_index].allows_entering_room(amphipod)
            }).collect_vec()
    }

    fn move_amphipod(&mut self, from: &Coords, to: &Coords) {
        let from_index = self._coords_to_index(from);
        let amphipod = self.amphipods[from_index].clone().unwrap();
        let to_index = self._coords_to_index(to);
        self.amphipods[from_index] = None;
        self.amphipods[to_index] = Some(amphipod.clone());

        let enters_room = from.y == 1 && to.y == 2;
        let leaves_room = from.y == 2 && to.y == 1;
        if enters_room {
            println!("enters room");
            let entrance = &mut self.entrances[from_index];
            *entrance = Entrance::ToColoredRoom(amphipod.color);
        } else if leaves_room {
            let is_room_empty = self.is_room_empty(from.x);
            let entrance = &mut self.entrances[to_index];
            if is_room_empty {
                *entrance = Entrance::ToEmptyRoom
            }
        }

        self.amphipods[to_index].as_mut().unwrap().must_move_next = leaves_room; // todo: consider this
    }

    fn free_spaces_around(&self, coords: &Coords) -> Vec<Coords> {
        vec![(-1,0), (1,0), (0,-1), (0,1)].into_iter()
            .map(|(x, y)| (coords.x+x, coords.y+y))
            .map(|(x, y)| Coords{x, y})
            .filter(|c| self.is_on_map(c))
            .filter(|c| {
                let i = self._coords_to_index(c);
                matches!(self.burrow[i], Tile::Floor) && self.amphipods[i].is_none()
            })
            .collect_vec()
    }

    fn is_on_map(&self, coords: &Coords) -> bool {
        coords.x>=0 && coords.x < self.width as isize
            && coords.y >= 0 && coords.y < self.height as isize
    }

    fn is_room_empty(&self, x: isize) -> bool {
        let one = self._coords_to_index(&Coords {x, y: 2});
        let two = self._coords_to_index(&Coords {x, y: 3});
        self.amphipods[one].is_none() && self.amphipods[two].is_none()
    }

    fn is_room_full(&self, x: isize) -> bool {
        let one = self._coords_to_index(&Coords {x, y: 2});
        let two = self._coords_to_index(&Coords {x, y: 3});
        self.amphipods[one].is_some() && self.amphipods[two].is_some()
    }


    fn is_solved(&self) -> bool {
        self.entrances.iter().filter(|e|matches!(e, Entrance::ToColoredRoom(_))).count() == 4
        //&& self.is_room_full(3) && self.is_room_full(5)
        //&& self.is_room_full(7) && self.is_room_full(9)
    }

    fn successors(&self) -> Vec<(Self, usize)> {
        self.amphipods.iter().enumerate()
            .filter(|(_, a)|a.is_some())
            .map(|(i, a)|(i, a.as_ref().unwrap()))
            .map(|(i, a)|(i, self.possible_moves_for(&i)))
            .flat_map(|(i, moves)| moves.into_iter()
                .map(move |m|{
                    let mut successor = self.clone();
                    successor.move_amphipod(&self._index_to_coords(&i), &m);
                    let cost = self.amphipods[i].as_ref().map(|a|a.energy_per_step).unwrap();
                    (successor, cost)
            }))
            .collect()
    }

    fn heuristics(&self) -> usize {
            todo!()

        //todo euklid dist to room
    }

}


fn solve(map: &Map, cache: &mut HashMap<u64, usize>) -> usize {
    println!("{:?}\n", map);
    if map.is_solved() {
        0
    } else {

        map.successors().iter().map(|(m, cost) | {
            let mut h = DefaultHasher::new();
            map.hash(&mut h);
            let key = h.finish();
            println!("{:?}\n {}", map, key);
            cost + if let Some(solution) = cache.get(&key) {
                println!("hit");
                *solution
            } else {
                let solution = solve(m, cache);
                cache.insert(key, solution);
                solution
            }
        }).min().unwrap()
    }
}


impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buffer: Vec<char> = vec![];
        self.burrow.chunks(self.width).for_each(|c| {
            buffer.extend(c.iter().flat_map(|t|format!("{}", t).chars().collect_vec()));
        });
        self.entrances.iter().enumerate().for_each(|(i, e)| {
            match e {
                Entrance::None => {}
                Entrance::ToMixedRoom => {buffer[i] = 'm';}
                Entrance::ToColoredRoom(c) => {buffer[i] = c.to_ascii_lowercase();}
                Entrance::ToEmptyRoom => {buffer[i] = 'e'}
            }
        });
        self.amphipods.iter()
            .enumerate()
            .flat_map(|(i,a)| a.as_ref().map(|a|(i, a)))
            .for_each(|(i, a)| {
                buffer[i] = a.label
            });

        write!(f, "{}", buffer.chunks(self.width)
            .map(|c| c.iter().join("")).join("\n"))
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <&Map as Display>::fmt(&self, f)
    }
}

impl Entrance {
    fn allows_entering_room(&self, a: &Amphipod) -> bool {
        match self {
            Entrance::None => false,
            Entrance::ToMixedRoom => false,
            Entrance::ToColoredRoom(color) => color == &a.color,
            Entrance::ToEmptyRoom => true,
        }
    }
}


#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use pathfinding::prelude::{bfs, dijkstra};
    use crate::{Amphipod, Map};

    #[test]
    fn test_build_map() {
        let tiles = "##############...........####B#C#B#D###  #A#D#C#A#    #########  ";
        let map = Map::from_tiles(&tiles.chars().collect_vec(), 13);
        println!("{}", map);
    }

    #[test]
    fn test_goal() {
        //let burrow = Burrow {
        //    room_color: vec![Some(0), Some(1), Some(2), Some(3)],
        //    amphipods: vec![
        //        Amphipod::new('A', 0, 1),
        //        Amphipod::new('A', 0, 1),
        //        Amphipod::new('B', 1, 10),
        //        Amphipod::new('B', 1, 10),
        //        Amphipod::new('C', 2, 100),
        //        Amphipod::new('C', 2, 100),
        //        Amphipod::new('D', 3, 1000),
        //        Amphipod::new('D', 3, 1000)
        //    ]
        //};
        //assert!(burrow.target_reached());

    }

    #[test]
    fn test_example_0() {

        //let burrow = Burrow {
        //    room_color: vec![None, None, None, None],
        //    amphipods: vec![
        //        Amphipod::new('A', 0, 1),
        //        Amphipod::new('A', 0, 1),
        //        Amphipod::new('B', 1, 10),
        //        Amphipod::new('B', 1, 10),
        //        Amphipod::new('C', 2, 100),
        //        Amphipod::new('C', 2, 100),
        //        Amphipod::new('D', 3, 1000),
        //        Amphipod::new('D', 3, 1000)
        //    ]
        //};
//

        //println!("///");
        //assert_eq!(4, burrow.successors().len());


        let tiles = "##############...........####B#C#B#D###  #A#D#C#A#    #########  ";
        let map = Map::from_tiles(&tiles.chars().collect_vec(), 13);


        let result = bfs(
            &map,
            |b| {
                //println!("{}", b);
                let s = b.successors();
                //println!("{:?}", s);
                s.into_iter().map(|(s, _)|s).collect_vec()
            },
            |b| b.is_solved()
        );

        println!("{:?}", result);
    }

}

