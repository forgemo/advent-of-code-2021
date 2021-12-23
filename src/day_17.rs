mod lib;

use std::usize;
use itertools::{Itertools};

fn main() {
    let target_x = (192,251);
    let target_y= (-89,-59);

    let task_a = task_a(target_y.0);
    assert_eq!(3916, task_a);

    let task_b = task_b(target_x, target_y);
    assert_eq!(2986, task_b);

    println!("task_a: {}, task_b: {}", task_a, task_b);
}

fn task_a(min_y: isize) -> isize {
    (min_y * (min_y + 1)) / 2
}

fn task_b(target_x: (isize, isize), target_y: (isize, isize)) -> usize {
    let launcher = Launcher { target_x, target_y };
    launcher.all_possible_hit_velocities().len()
}


struct Launcher {
    target_x: (isize, isize),
    target_y: (isize, isize),
}

impl Launcher {
    fn will_hit_target(&self, initial_v: (isize, isize)) -> Option<(isize, isize)> {
        let (mut v, mut pos) = (initial_v, (0, 0));
        loop {
            pos.0 += v.0; pos.1 += v.1;
            v.0 = (v.0 - 1).max(0); v.1 -= 1;

            let out_of_range = pos.0 > self.target_x.1 || pos.1 < self.target_y.0;

            if self.in_target(pos) {  return Some(initial_v); }
            else if out_of_range { return None; }
        }
    }

    fn will_hit_x(&self, initial_v: isize) -> Option<isize> {
        let (mut v, mut pos) = (initial_v, 0);
        loop {
            pos += v; v = (v - 1).max(0);
            if self.is_x_hit(pos) { return Some(initial_v); }
            else if v == 0 || pos > self.target_x.1 { return None; }
        }
    }

    fn will_hit_y(&self, initial_v: isize) -> Option<isize> {
        let (mut v, mut pos) = (initial_v, 0);
        loop {
            pos += v; v -= 1;
            if self.is_y_hit(pos) { return Some(initial_v); }
            else if pos < self.target_y.0 { return None; }
        }
    }

    fn all_possible_hit_velocities(&self) -> Vec<(isize, isize)> {
        let x_hits =
            (1..self.target_x.1 + 1).filter_map(|x| self.will_hit_x(x));
        let y_hits =
            (self.target_y.0..self.target_y.0 * -1 + 1).filter_map(|y| self.will_hit_y(y));

        x_hits.cartesian_product(y_hits)
        .filter_map(|v| self.will_hit_target(v))
        .collect()
    }


    fn is_x_hit(&self, x: isize) -> bool {
        x >= self.target_x.0 && x <= self.target_x.1
    }

    fn is_y_hit(&self, y: isize) -> bool {
        y >= self.target_y.0 && y <= self.target_y.1
    }

    fn in_target(&self, p: (isize, isize)) -> bool {
        self.is_x_hit(p.0) && self.is_y_hit(p.1)
    }
}
