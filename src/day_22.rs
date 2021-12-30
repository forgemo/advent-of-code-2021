use std::collections::HashSet;
use itertools::{Itertools};
use regex::Regex;
use crate::lib::read_lines;

mod lib;

fn main() {
    let task_a = task_a(read_lines("input/day_22.txt"));
    assert_eq!(580810, task_a);
    let task_b = task_b(read_lines("input/day_22.txt"));
    assert_eq!(1265621119006734, task_b);
    println!("task_a: {}, task_b: {}", task_a, task_b);
}

fn task_a(lines: impl Iterator<Item=String>) -> isize {
    let commands = parse_input(lines).into_iter()
        .filter(|(_, c)|
            c.from.x <= 50 && c.to.x >= -50
            && c.from.y <= 50 && c.to.y >= -50
            && c.from.z <= 50 && c.to.z >= -50
        )
        .collect_vec();
    let mut reactor = Reactor::new();
    reactor.perform_all(&commands);
    reactor.count_cubes()
}

fn task_b(lines: impl Iterator<Item=String>) -> isize {
    let commands = parse_input(lines);
    let mut reactor = Reactor::new();
    reactor.perform_all(&commands);
    reactor.count_cubes()
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct Cube {x:isize, y:isize, z:isize}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct Cuboid {
    from: Cube,
    to: Cube,
}

struct Reactor {
    active_cuboids: HashSet<Cuboid>,
}

impl Cuboid {

    fn from_ranges(x:(isize, isize), y: (isize, isize), z: (isize, isize)) -> Self {
        let from = Cube{x: x.0, y: y.0, z: z.0};
        let to = Cube{x: x.1, y: y.1, z: z.1};
        Self {from, to}
    }

    fn collides_with(&self, other: &Cuboid) -> bool {
        (self.center_x() - other.center_x()).abs() < self.len_x() as f64/2. + other.len_x() as f64 /2.
            && (self.center_y() - other.center_y()).abs() < self.len_y() as f64/2. + other.len_y() as f64/2.
            && (self.center_z() - other.center_z()).abs() < self.len_z() as f64/2. + other.len_z() as f64/2.
    }

    fn center_x(&self) -> f64 {self.from.x as f64 + (self.to.x as f64 - self.from.x as f64)/2.}
    fn center_y(&self) -> f64 {self.from.y as f64 + (self.to.y as f64 - self.from.y as f64)/2.}
    fn center_z(&self) -> f64 {self.from.z as f64 + (self.to.z as f64 - self.from.z as f64)/2.}
    fn len_x(&self) -> isize {self.to.x - self.from.x + 1}
    fn len_y(&self) -> isize {self.to.y - self.from.y + 1}
    fn len_z(&self) -> isize {self.to.z - self.from.z + 1}
    fn intersection_x(&self, other: &Cuboid) -> (isize, isize) {
        (self.from.x.max(other.from.x), self.to.x.min(other.to.x))
    }
    fn intersection_y(&self, other: &Cuboid) -> (isize, isize) {
        (self.from.y.max(other.from.y), self.to.y.min(other.to.y))
    }
    fn intersection_z(&self, other: &Cuboid) -> (isize, isize) {
        (self.from.z.max(other.from.z), self.to.z.min(other.to.z))
    }

    fn intersection(&self, other: &Cuboid) -> Option<Cuboid> {
        if self.collides_with(other) {
            Some(Cuboid::from_ranges(
                self.intersection_x(other),
                self.intersection_y(other),
                self.intersection_z(other),
            ))
        } else {
            None
        }
    }

    fn encloses(&self, other: &Cuboid) -> bool {
        self.from.x <= other.from.x && self.to.x >= other.to.x
            && self.from.y <= other.from.y && self.to.y >= other.to.y
            && self.from.z <= other.from.z && self.to.z >= other.to.z
    }

    fn volume(&self) -> isize {
        self.len_x() * self.len_y() * self.len_z()
    }

    fn split_around(&self, intersection: &Cuboid) -> HashSet<Cuboid> {
        let i = intersection;
        let x_cuts = vec![(self.from.x, i.from.x-1), (i.from.x, i.to.x), (i.to.x+1, self.to.x)];
        let y_cuts = vec![(self.from.y, i.from.y-1), (i.from.y, i.to.y), (i.to.y+1, self.to.y)];
        let z_cuts = vec![(self.from.z, i.from.z-1), (i.from.z, i.to.z), (i.to.z+1, self.to.z)];

        x_cuts.iter().flat_map(|x| {
            y_cuts.iter().flat_map( |y| {
                z_cuts.iter()
                    .filter(|z| x.1-x.0 >= 0 && y.1-y.0 >= 0 && z.1-z.0 >= 0)
                    .map( |z| {
                    Cuboid::from_ranges((x.0, x.1), (y.0, y.1), (z.0, z.1))
                })
            })
        })
        .collect()
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
enum Action {
    On, Off
}

impl Reactor {

    fn new() -> Self {
        Self {
            active_cuboids: Default::default()
        }
    }

    fn perform(&mut self, action: &Action, cuboid: &Cuboid ) {
        let intersections = self.active_cuboids.iter()
            .cloned()
            .filter_map(|c|cuboid.intersection(&c).map(|i|(c,i)))
            .collect_vec();

        for  (intersecting_cuboid, intersection) in intersections {
            if cuboid.encloses(&intersecting_cuboid) {
                self.active_cuboids.remove(&intersecting_cuboid);
            } else {
                let mut parts = intersecting_cuboid.split_around(&intersection);
                parts.remove(&intersection);
                self.active_cuboids.remove(&intersecting_cuboid);
                self.active_cuboids.extend(parts.into_iter());
            }
        }
        if action == &Action::On {
            self.active_cuboids.insert(cuboid.clone());
        }
    }

    fn perform_all(&mut self, actions: &[(Action, Cuboid)] ) {
       actions.iter().for_each(|(a, c)| {
           self.perform(a, c);
       })
    }

    fn count_cubes(&self) -> isize {
        self.active_cuboids.iter().map(|c|c.volume()).sum()
    }


}

fn parse_input(lines: impl Iterator<Item=String>) -> Vec<(Action, Cuboid)> {

    let re = Regex::new(r"^(.+?) x=(.+?)\.\.(.+?),y=(.+?)\.\.(.+?),z=(.+?)\.\.(.+?)$").unwrap();

    lines.map(|line|{
        let cap = re.captures_iter(line.as_str()).next().unwrap();
        let mut i = cap.iter().skip(1);
        let command = i.next().unwrap().unwrap().as_str();
        let ranges = i.map(|v|v.unwrap().as_str().parse::<isize>().unwrap()).tuples().collect_vec();
        let action = if command == "on" {Action::On} else {Action::Off};
        (action, Cuboid::from_ranges(ranges[0], ranges[1], ranges[2]))
    }).collect_vec()
}


#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::{Cube, Cuboid, Reactor};
    use crate::Action::{Off, On};

    #[test]
    fn test_example_0() {
        let a = Cuboid::from_ranges((10,12),(10,12),(10,12));
        assert_eq!(Cube{x: 10, y: 10, z: 10}, a.from);
        assert_eq!(Cube{x: 12, y: 12, z: 12}, a.to);
        assert_eq!(27, a.volume());

        let mut reactor = Reactor::new();
        reactor.perform(&On, &Cuboid::from_ranges((10, 12),(10, 12), (10,12)));
        assert_eq!(27, reactor.count_cubes());
        reactor.perform(&On, &Cuboid::from_ranges((11, 13),(11, 13), (11,13)));
        assert_eq!(27+19,  reactor.count_cubes());
        reactor.perform(&Off, &Cuboid::from_ranges((9, 11),(9, 11), (9,11)));
        assert_eq!(27+19-8, reactor.count_cubes());
        reactor.perform(&On, &Cuboid::from_ranges((10, 10),(10, 10), (10,10)));
        assert_eq!(27+19-8+1,  reactor.count_cubes());
        assert_eq!(39,  reactor.count_cubes());
    }

    #[test]
    fn test_collision() {
        let a = Cuboid::from_ranges((0,2), (0, 2), (0, 2));
        let b = Cuboid::from_ranges((0,1), (0, 1), (0, 1));
        let c = Cuboid::from_ranges((2,3), (2, 3), (2, 3));
        let e = Cuboid::from_ranges((3,4), (3, 4), (3, 4));
        assert_eq!(2.5, c.center_x());
        assert_eq!(2.5, c.center_y());
        assert_eq!(2.5, c.center_z());
        assert_eq!(2, c.len_x());
        assert_eq!(2, c.len_y());
        assert_eq!(2, c.len_z());
        assert!(a.collides_with(&b));
        assert!(a.collides_with(&c));
        assert!(!a.collides_with(&e));
    }

    #[test]
    fn test_intersection() {
        let a = Cuboid::from_ranges((0,5), (0, 5), (0, 5));
        let b = Cuboid::from_ranges((1,6), (1, 6), (1, 6));
        let c = Cuboid::from_ranges((1,5), (1, 5), (1, 5));
        assert_eq!(c, a.intersection(&b).unwrap());
    }


    #[test]
    fn test_split_a() {
        let a = Cuboid::from_ranges((0,5), (0, 5), (0, 5));
        let b = Cuboid::from_ranges((1,2), (1, 2), (1, 2));
        let parts = a.split_around(&b);
        println!("{:#?} {:?}", parts, parts.iter().map(|p|p.volume()).collect_vec());

        assert_eq!(27, parts.len());
    }

    #[test]
    fn test_split_b() {
        let a = Cuboid::from_ranges((0,5), (0, 5), (0, 5));
        let b = Cuboid::from_ranges((0,5), (0, 5), (0, 2));
        let parts = a.split_around(&b);
        assert_eq!(2, parts.len());
        assert_eq!(216, a.volume());
        assert_eq!(216, parts.iter().map(|p|p.volume()).sum::<isize>());
    }
}

