use itertools::Itertools;

use crate::lib::read_lines;

mod lib;

fn main() {
    let task_a = task_a(read_lines("input/day_18.txt"));
    assert_eq!(4111, task_a);

    let task_b = task_b(read_lines("input/day_18.txt"));
    assert_eq!(4917, task_b);

    println!("task_a: {}, task_b: {}", task_a, task_b);
}

fn task_a(lines: impl Iterator<Item=String>) -> isize {
    let sum = parse_lines(lines)
        .into_iter()
        .reduce(|a,b| add(&a, &b))
        .expect("reduce failed");
    Magnitude {n: Box::new(sum.into_iter())} .calculate()
}

fn task_b(lines: impl Iterator<Item=String>) -> isize {
    let sum = parse_lines(lines);
    sum.iter().cartesian_product(sum.iter())
        .filter(|(a,b)|a!=b)
        .flat_map(|(a, b)| vec![add(a, b), add(b, a)])
        .map(|n|reduce(&n))
        .map(|n|Magnitude {n: Box::new(n.into_iter())} .calculate())
        .max().unwrap()
}

type Number = Vec<isize>;

fn parse_lines(lines: impl Iterator<Item=String>) -> Vec<Number> {
    lines.map(|l| parse_line(&l)).collect()
}

fn parse_line(line: &str) -> Number {
    line.chars().map(|c| match c {
        '[' => -1,
        ']' => -2,
        ',' => -3,
        _ => c.to_digit(10).unwrap() as isize
    }).collect()
}

fn add(lhs: &Number, rhs: &Number) -> Number {
    let sum = vec![vec![-1],lhs.clone(), vec![-3], rhs.clone(), vec![-2]]
        .concat();
    reduce(&sum)
}

fn explode(n: &Number) -> Number {

    let mut n = n.clone();
    let most_left = n.iter().scan((0,0), |s ,v| {
        match v { -1 => s.1 += 1, -2 => s.1 -= 1, _ => {} };
        let r = Some(s.clone());
        s.0 +=1;
        r
    })
        .filter(|(_, s)| *s==5)
        .map(|(i, _)| i)
        .next();

    if let Some(i) = most_left {
        let left = n[i+1];
        let right = n[i+3];
        n.remove(i);
        n.remove(i);
        n.remove(i);
        n.remove(i);
        n[i] = 0;
        for i in (0..i).rev() {
            if n[i] >= 0 {
                n[i] += left;
                break;
            }
        }
        for i in i+1..n.len() {
            if n[i] >= 0 {
                n[i] += right;
                break;
            }
        }
    }
    n
}

fn reduce(n: &Number) -> Number {
    let mut last = n.clone();

    loop {
        let e = explode(&last);
        if e != last {
            last = e;
            continue
        }
        let s = split(&last);
        if s!=last {
            last = s;
            continue
        }
        break;
    }
    last
}

fn split(n: &Number) -> Number {
    n.iter().scan((false, false), |s ,v| {
        s.0 = !s.1 && *v >= 10;
        s.1 = s.1 || s.0;
        Some((s.0,v))
    }).map(|(s, v)| {
        if s {
            vec![-1, (*v as f64 / 2.).floor() as isize, -3,
                 (*v as f64 / 2.).ceil() as isize, -2]
        } else {
            vec![*v]
        }
    }
    ).flatten().collect()
}


struct Magnitude {
    n: Box<dyn Iterator<Item=isize>>
}

impl Magnitude {
    fn calculate(&mut self) -> isize {
        match self.n.next() {
            Some(-1) => {
                let left = self.calculate();
                self.n.next().unwrap();
                let right= self.calculate();
                self.n.next().unwrap();
                3 * left + 2 * right
            },
            Some(v) => v,
            _ => panic!("not expected")
        }
    }
}

fn print(n: &Number)  {
    n.iter()
        .for_each(|c| match c {
            -1 => print!("["),
            -2 => print!("]"),
            -3 => print!(","),
            v => print!("{}", v)
        });
    println!()
}
