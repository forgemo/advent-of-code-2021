use std::ops::Index;
use itertools::{Itertools};

use crate::lib::read_lines;

mod lib;

fn main() {
    let report = Report::parse(read_lines("input/day_19.txt")).normalize();
    let task_a = report.unique_beacons().len();
    let task_b= report.max_manhattan_distance();
    assert_eq!(376, task_a);
    assert_eq!(10772, task_b);
    println!("task_a: {}, task_b: {}", task_a, task_b);
}

#[derive(Debug, Clone)]
struct Report {
    scanners: Vec<Scanner>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct Position {
    x: isize,
    y: isize,
    z: isize,
}

#[derive(Debug, Clone)]
struct Scanner {
    id: usize,
    _beacons: Vec<Beacon>,
    orientation: Orientation,
    translation: Translation
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct Beacon {
    position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Orientation {
    x_flip: isize,
    y_flip: isize,
    z_flip: isize,
    x_axis_idx: usize,
    y_axis_idx: usize,
    z_axis_idx: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Translation {
    x: isize,
    y: isize,
    z: isize,
}

impl From<Vec<isize>> for Position {
    fn from(v: Vec<isize>) -> Self {
        Self { x: v[0], y: v[1], z: v[2] }
    }
}

impl Index<usize> for Position {
    type Output = isize;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("index out of bounds")
        }
    }
}

impl Orientation {
    fn possible_orientations() -> Vec<Self> {
        let flip_permutations = vec![
            vec![ 1, 1, 1], vec![ 1,-1,-1], vec![-1,-1, 1], vec![-1, 1,-1],
            vec![-1,-1,-1], vec![ 1, 1,-1], vec![ 1,-1, 1], vec![-1, 1, 1],
        ];
        let axis_permutations = vec![0, 1, 2].into_iter().permutations(3).collect_vec();
        flip_permutations.iter().map(|flip| axis_permutations.iter().map(|axis| {
            Orientation {
                x_flip: flip[0],
                y_flip: flip[1],
                z_flip: flip[2],
                x_axis_idx: axis[0],
                y_axis_idx: axis[1],
                z_axis_idx: axis[2],
            }
        })).flatten().collect()
    }

    fn identity() -> Orientation {
        Orientation {
            x_flip: 1,
            y_flip: 1,
            z_flip: 1,
            x_axis_idx: 0,
            y_axis_idx: 1,
            z_axis_idx: 2,
        }
    }
}

impl Translation {

    fn new(x:isize, y:isize, z:isize) -> Self {
        Self{x, y, z}
    }
    fn identity() -> Self {
        Self::new(0, 0, 0)
    }

    fn add(&self, rhs: &Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Position {
    fn translation_to(&self, other: &Self) -> Translation {
        Translation::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    fn translate(&self, t: &Translation) -> Self {
        Self { x: t.x + self.x, y: t.y + self.y, z: t.z + self.z }
    }
}

impl Beacon {

    fn apply_orientation(&self, o: &Orientation) -> Beacon {
        Beacon { position: Position {
            x: self.position[o.x_axis_idx] * o.x_flip,
            y: self.position[o.y_axis_idx] * o.y_flip,
            z: self.position[o.z_axis_idx] * o.z_flip
        }}
    }

    fn apply_translation(&self, t: &Translation) -> Beacon {
        Beacon { position: self.position.translate(t)}
    }
}


impl Scanner {
    fn new(id: usize, beacons: Vec<Beacon>) -> Self {
        Self { id, _beacons: beacons, orientation: Orientation::identity(), translation: Translation::identity() }
    }

    fn beacons(&self) -> Vec<Beacon> {
        self._beacons.iter().map(|b| b
            .apply_orientation(&self.orientation)
            .apply_translation(&self.translation)
        ).collect()
    }

    fn add_translation(&self, t: &Translation) -> Self {
        Scanner {translation: self.translation.add(t), ..self.clone()}
    }

    fn with_orientation(&self, orientation: Orientation) -> Self {
        Scanner {orientation, ..self.clone()}
    }

    fn overlap_with(&self, other: &Self) -> Option<(Vec<Beacon>, Scanner)>{
        let min_match_count = 12;
        Orientation::possible_orientations().into_iter()
            .map(|orientation| other.with_orientation(orientation))
            .flat_map(|other_scanner| {
                other_scanner.beacons().iter().map(|other_beacon| {
                    let self_beacons = self.beacons();
                    self_beacons.iter().map(|b| {
                        let translation = other_beacon.position.translation_to(&b.position);
                        let other = other_scanner.add_translation(&translation);
                        let matches = other.beacons().into_iter()
                            .filter(|o| self_beacons.iter().contains(o))
                            .collect_vec();
                        (matches, other)
                    })
                    .find(|(m, _)| m.len() >= min_match_count)
                }).find(|m|m.is_some())
            }).next().flatten()
    }
}


impl Report {
    fn parse(lines: impl Iterator<Item=String>) -> Self {
        let mut lines = lines.peekable();

        let mut scanners = vec![];
        while lines.peek().is_some() {
            let mut scanner_lines = lines.by_ref().take_while(|l| !l.is_empty());
            let id = scanner_lines.next().unwrap()
                .split_whitespace().nth(2).unwrap()
                .parse::<usize>().unwrap();

            let beacons = scanner_lines
                .map(|l| l.split(',').map(|c| c.parse::<isize>().unwrap()).collect_vec())
                .map(|v| Beacon { position: Position::from(v) })
                .collect();

            scanners.push(Scanner::new(id, beacons));
        }
        Self { scanners }
    }

    fn normalize(&self) -> Self {
        let mut scanners = self.scanners.clone();
        let mut normalized_scanners = vec![scanners.pop().unwrap()];

        while let Some(scanner) = scanners.pop() {
            let mut had_match = false;
            for norm_scanner in normalized_scanners.clone() {
                if norm_scanner.id == scanner.id {
                    continue
                }
                match norm_scanner.overlap_with(&scanner) {
                    None => {}
                    Some((_, s)) => {
                        println!("matched norm {} with {}", norm_scanner.id, scanner.id);
                        had_match = true;
                        normalized_scanners.push(s.clone());
                        break
                    }
                }
            }
            if !had_match {
                scanners.insert(0, scanner);
            }
        }
        Self{scanners: normalized_scanners}
    }

    fn unique_beacons(&self) -> Vec<Beacon> {
        self.scanners.iter().flat_map(|s|s.beacons()).sorted().dedup().collect_vec()
    }

    fn max_manhattan_distance(&self) -> usize {
        self.scanners.iter().map(|s|&s.translation).combinations(2)
            .map(|v| (v[0].x-v[1].x).abs() + (v[0].y-v[1].y).abs() + (v[0].z-v[1].z).abs())
            .max().unwrap() as usize
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::{Orientation, Position, read_lines, Report, Scanner, Translation};


    #[test]
    fn test_example_0_1() {
        let report = Report::parse(read_lines("input/day_19_test.txt"));
        let (matches, scanner_1_rel_to_0) = report.scanners[0].overlap_with(&report.scanners[1]).unwrap();
        let matches = matches.iter().flat_map(|m|vec![m.position.x, m.position.y, m.position.z]).collect_vec();
        assert_eq!(matches, vec![-618,-824,-621, -537,-823,-458, -447,-329,318, 404,-588,-901, 544,-627,-890, 528,-643,409, -661,-816,-575, 390,-675,-793, 423,-701,434, -345,-311,381, 459,-707,401, -485,-357,347]);
        assert_eq!(Translation{x: 68, y: -1246, z: -43},scanner_1_rel_to_0.translation);
        assert!(report.scanners[1].overlap_with(&report.scanners[0]).is_some());
    }

    #[test]
    fn test_example_1_3() {
        let report = Report::parse(read_lines("input/day_19_test.txt"));
        let (_, scanner_1_rel_to_0) = report.scanners[0].overlap_with(&report.scanners[1]).unwrap();
        let (m, s) = scanner_1_rel_to_0.overlap_with(&report.scanners[3]).unwrap();
        println!("{:?}", s.orientation);
        println!("{:?}", s.translation);
        println!("{}", m.len());
        report.scanners[3].overlap_with(&report.scanners[1]).unwrap();
    }

    #[test]
    fn test_example_4_1() {
        let report = Report::parse(read_lines("input/day_19_test.txt"));
        let (_, s) = report.scanners[1].overlap_with(&report.scanners[4]).unwrap();
        let (_, s) = report.scanners[4].overlap_with(&report.scanners[1]).unwrap();
        println!("{:?}", s.orientation);
        println!("{:?}", s.translation);
    }

    #[test]
    fn test_example_2_4() {
        let report = Report::parse(read_lines("input/day_19_test.txt"));
        let (_, s) = report.scanners[4].overlap_with(&report.scanners[2]).unwrap();
        let (_, s) = report.scanners[2].overlap_with(&report.scanners[4]).unwrap();
        println!("{:?}", s.orientation);
        println!("{:?}", s.translation);
    }


    #[test]
    fn test_example_0_1_2_3_4() {
        let report = Report::parse(read_lines("input/day_19_test.txt"));
        let n0 = report.scanners[0].clone();
        let (m1, n1) = report.scanners[0].overlap_with(&report.scanners[1]).unwrap();
        let (m2, n3) = n1.overlap_with(&report.scanners[3]).unwrap();
        let (m3, n4) = n1.overlap_with(&report.scanners[4]).unwrap();
        let (m4, n2) = n4.overlap_with(&report.scanners[2]).unwrap();
        assert_eq!(Translation::identity(), n0.translation);
        assert_eq!(Translation::new(68,-1246,-43), n1.translation);
        assert_eq!(Translation::new(1105,-1205,1229), n2.translation);
        assert_eq!(Translation::new(-92,-2380,-20), n3.translation);
        assert_eq!(Translation::new(-20,-1133,1061 ), n4.translation);
        assert_eq!(12, m1.len());
        assert_eq!(12, m2.len());
        assert_eq!(12, m3.len());
        assert_eq!(12, m4.len());
        let beacons = vec![n0.beacons(), n1.beacons(), n2.beacons(), n3.beacons(), n4.beacons()].concat()
            .into_iter().sorted().dedup().collect_vec();
        assert_eq!(79, beacons.len());
    }

    #[test]
    fn test_normalize() {
        let report = Report::parse(read_lines("input/day_19_test.txt")).normalize();
        assert_eq!(79, report.unique_beacons().len());
    }

}

