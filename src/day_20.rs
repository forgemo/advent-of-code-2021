use std::iter::once;
use std::usize;
use itertools::{Itertools};

use crate::lib::read_lines;

mod lib;


fn main() {
    let task_a = task(read_lines("input/day_20.txt"), 2);
    let task_b = task(read_lines("input/day_20.txt"), 50);
    assert_eq!(5479, task_a);
    assert_eq!(19012, task_b);
    println!("task_a: {}, task_b: {}", task_a, task_b);
}

fn task(mut lines: impl Iterator<Item=String>, enhancement_count: usize) -> usize {
    let algo = lines.next().unwrap().chars().collect();
    let mut lines = lines.skip(1).peekable();
    let width = lines.peek().unwrap().chars().count();
    let pixels = lines.flat_map(|l|l.chars().collect_vec()).collect();
    let mut image = Image::new(pixels, width as isize, algo);
    image.print();
    for i in 0..enhancement_count {
        println!("enhancement: {}", i);
        image = image.enhance();
    }
    image.print().count_lit_pixels()
}


type Coords = (isize, isize);

struct Image {
    origin: Coords,
    input_width: isize,
    input_height: isize,
    pixels: Vec<char>,
    algo: Vec<char>,
    is_infinity_lit: bool
}

impl Image {
    fn new(pixels: Vec<char>, width: isize, algo: Vec<char>) -> Self {
        let height = pixels.len() as isize/width;
        Image {
            pixels,
            input_width: width,
            input_height: height,
            algo ,
            origin: (0, 0),
            is_infinity_lit: false
        }
    }

    fn coords_to_index(&self, coords: Coords) -> Option<usize> {
        let (x, y) = (coords.0 - self.origin.0, coords.1 - self.origin.1);
        if x < 0 || y < 0 || x >= self.input_width || y >= self.input_height {
            None
        } else {
            Some((y * self.input_width as isize + x) as usize)
        }
    }

    fn pixel_at(&self, coords: Coords) -> &char {
        self.coords_to_index(coords)
            .and_then(|i|self.pixels.get(i))
            .unwrap_or_else(||self.infinity_pixel())
    }
    fn coords_around(&self, coords: Coords) -> Vec<Coords> {
        let (x, y) = coords;
        (y-1..y+2).flat_map(|y| (x-1..x+2).map(|x|(x, y)).collect_vec()).collect()
    }

    fn visit_all_coords(&self) -> impl Iterator<Item=Coords> + '_ {
        self.visit_all_rows().flatten()
    }

    fn visit_all_rows(&self) -> impl Iterator<Item=Vec<Coords>> + '_ {
        let (origin_x, origin_y) = self.origin;
        (origin_y..(origin_y+self.input_height))
            .map(move |y| (origin_x..(origin_x+self.input_width))
                .map(move |x| (x as isize, y as isize)).collect_vec())
    }

    fn visit_infinitely_circling_around(&self) -> impl Iterator<Item=(isize, Coords)> {
        let x = self.input_width as isize / 2;
        let y = self.input_height as isize / 2;
        let cycle_iter = [0, 1, 2, 3].into_iter()
            .cycle()
            .enumerate()
            .map(|(i, side )| ((i/4) as isize + 1, side))
            .flat_map(move |(cycle, side)| {
                match side {
                    0 => {(x-cycle..x+cycle).map(|x|(cycle, (x, y-cycle)))}.collect_vec(),
                    1 => {(y-cycle..y+cycle).map(|y|(cycle, (x+cycle, y)))}.collect_vec(),
                    2 => {(x-cycle+1..x+cycle+1).rev().map(|x|(cycle, (x, y+cycle)))}.collect_vec(),
                    3 => {(y-cycle+1..y+cycle+1).rev().map(|y|(cycle, (x-cycle, y)))}.collect_vec(),
                    _ => panic!("unexpected side {}", side)
                }
            });

            once((0, (x, y))).chain(cycle_iter)
    }

    fn pixels_at(&self, coords: &Vec<Coords>) -> Vec<char> {
        coords.iter().map(|&coords| *self.pixel_at(coords)).collect()
    }

    fn algo_index_from(&self, coord: Coords) -> usize {
        let relevant_coords = self.coords_around(coord);

        let relevant_pixels = self.pixels_at(&relevant_coords);
        let binary_string = relevant_pixels.iter()
            .map(|p| match p { '#' => '1', _  => '0' })
            .join("");
        usize::from_str_radix(&binary_string, 2).unwrap()
    }

    fn output_pixel_at(&self, coord: Coords) -> &char {
        let index = self.algo_index_from(coord);
        &self.algo[index]
    }

    fn enhance(&self) -> Self {
        let enhanced_pixels = self.visit_infinitely_circling_around()
            .map(|(cycle, coord)|( cycle, self.output_pixel_at(coord), coord))
            .scan((0, 4, '_'), |(started_cycle, keep_cycling, last_char), (cycle, pixel, coord)| {
                if *started_cycle != cycle {

                    if *keep_cycling > 0 {
                        *started_cycle = cycle;
                        *keep_cycling -= 1;
                    } else {
                        return None;
                    }
                }
                if pixel != last_char {
                    *last_char = *pixel;
                    *keep_cycling = 4
                }

                Some((*pixel, coord))
            })
            .sorted_by_key(|(_, (x, y))|(*y, *x))
            .collect_vec();

        let (_, (tlx, tly)) = enhanced_pixels[0];
        let (_, (brx, bry)) = enhanced_pixels[enhanced_pixels.len()-1];

        let is_infinity_lit = (!self.is_infinity_lit && self.flips_infinity_on())
            || (self.is_infinity_lit && !self.flips_infinity_off());

        Self {
            input_width: brx - tlx + 1,
            input_height: bry - tly + 1,
            pixels: enhanced_pixels.into_iter().map(|(p, _)|p).collect_vec(),
            algo: self.algo.clone(),
            origin: (self.origin.0, self.origin.1),
            is_infinity_lit,
        }
    }

    fn print(&self) -> &Self {
        self.visit_all_rows().for_each(|row| {
           println!("{}", self.pixels_at(&row).iter().join(""));
        });
        self
    }

    fn count_lit_pixels(&self) -> usize {
        self.visit_all_coords()
            .map(|c|self.pixel_at(c))
            .filter(|p| p == &&'#')
            .count()
    }

    fn flips_infinity_on(&self) -> bool {
        self.algo[0] == '#'
    }

    fn flips_infinity_off(&self) -> bool {
        self.algo[self.algo.len()-1] == '.'
    }

    fn infinity_pixel(&self) -> &char {
        if self.is_infinity_lit {
            &'#'
        } else {
            &'.'
        }
    }

}


#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::Image;

    #[test]
    fn test_example_0() {
        let algo = vec![
            "..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..##",
             "#..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###",
             ".######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#.",
             ".#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#.....",
             ".#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#..",
             "...####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.....",
             "..##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#"
        ].concat();

        assert_eq!(512, algo.chars().count());

        let input_image = "#..#.\
                                #....\
                                ##..#\
                                ..#..\
                                ..###";

        let image = Image::new(
            input_image.chars().collect(), 5,
            algo.chars().collect()
        );

        assert_eq!(&'#', image.pixel_at((0, 0)));
        assert_eq!(&'.', image.pixel_at((4, 0)));
        assert_eq!(&'.', image.pixel_at((0, 4)));
        assert_eq!(&'#', image.pixel_at((4, 4)));
        assert_eq!(5, image.input_height);

        let relevant_pixels = vec![(4, 9), (5, 9), (6, 9), (4, 10), (5, 10), (6, 10), (4, 11), (5, 11), (6, 11)];
        assert_eq!(relevant_pixels, image.coords_around((5, 10)));

        let relevant_pixels = vec![(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (3, 2), (1, 3), (2, 3), (3, 3)];
        assert_eq!(relevant_pixels, image.coords_around((2, 2)));

        assert_eq!("...#...#.", image.pixels_at(&relevant_pixels).iter().join(""));
        assert_eq!(34, image.algo_index_from((2,2)));
        assert_eq!(&'#', image.output_pixel_at((2,2)));

        println!("-----------enhance 0------------");
        image.print();
        println!("-----------enhance 1------------");
        image.enhance().print();
        println!("-----------enhance 2------------");
        image.enhance().enhance().print();

        assert_eq!(35, image.enhance().enhance().count_lit_pixels());

        let mut image = Image::new(
            input_image.chars().collect(), 5,
            algo.chars().collect()
        );

        for i in 0..50 {
            println!("-----------enhance  {} ------------", i);
            image = image.enhance();
            image.print();
        }
        assert_eq!(3351, image.print().count_lit_pixels());

    }

}

