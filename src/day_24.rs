use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, format, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Mul, Sub};
use std::slice::SliceIndex;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use itertools::{Itertools};
use pathfinding::prelude::{astar, bfs};
use rayon::iter::plumbing::{Consumer, ProducerCallback, UnindexedConsumer};
use rayon::prelude::{ParallelIterator, IntoParallelIterator, IntoParallelRefIterator, IndexedParallelIterator};
use regex::Regex;
use crate::Instruction::*;
use crate::lib::{read_lines, read_usize_vec};
use crate::Value::Number;

mod lib;


fn main() {
    formula();

    let task_a = task_a(read_lines("input/day_24.txt"));
    assert_eq!(580810, task_a);
    let task_b = task_b(read_lines("input/day_24.txt"));
    assert_eq!(1265621119006734, task_b);
    println!("task_a: {}, task_b: {}", task_a, task_b);
}


fn task_a(lines: impl Iterator<Item=String>) -> usize {
    let result = find_star();
    println!("{:?}", result);

    todo!()
}

fn task_b(lines: impl Iterator<Item=String>) -> isize {
    todo!()
}


fn parse_input(lines: impl Iterator<Item=String>) -> Vec<Instruction> {
    lines.map(|line| {
        let parts = line.split_whitespace().collect_vec();
        match parts[0] {
            "inp" => Instruction::Inp(parse_var(parts[1])),
            "add" => Instruction::Add(parse_var(parts[1]), parse_value(parts[2])),
            "mul" => Instruction::Mul(parse_var(parts[1]), parse_value(parts[2])),
            "div" => Instruction::Div(parse_var(parts[1]), parse_value(parts[2])),
            "mod" => Instruction::Mod(parse_var(parts[1]), parse_value(parts[2])),
            "eql" => Instruction::Eql(parse_var(parts[1]), parse_value(parts[2])),
            _ => panic!("unexpected instruction {}", parts[0])
        }
    })
        .collect_vec()
}

fn parse_var(p: &str) -> Pointer {
    match p {
        "w" => Pointer::W,
        "x" => Pointer::X,
        "y" => Pointer::Y,
        "z" => Pointer::Z,
        _ => panic!("unexpected pointer {}", p)
    }
}

fn parse_value(p: &str) -> Value {
    match p {
        "w" | "x" | "y" | "z" => Value::Var(parse_var(p)),
        _ => Value::Number(p.parse::<isize>().unwrap())
    }
}

#[derive(Clone, Debug)]
enum Value {
    Number(isize),
    Var(Pointer),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Var(p) => write!(f, "{}", p)
        }
    }
}

#[derive(Clone, Debug)]
enum Pointer {
    W,
    X,
    Y,
    Z,
}

impl Pointer {
    fn index(&self) -> usize {
        match self {
            Pointer::W => 0,
            Pointer::X => 1,
            Pointer::Y => 2,
            Pointer::Z => 3,
        }
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Pointer::W => "w",
            Pointer::X => "x",
            Pointer::Y => "y",
            Pointer::Z => "z"
        })
    }
}


#[derive(Clone, Debug)]
enum Instruction {
    Inp(Pointer),
    Add(Pointer, Value),
    Mul(Pointer, Value),
    Div(Pointer, Value),
    Mod(Pointer, Value),
    Eql(Pointer, Value),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Inp(a) => write!(f, "{}=input", a),
            Instruction::Add(a, b) => write!(f, "{} + {}", a, b),
            Instruction::Mul(a, b) => write!(f, "{} * {}", a, b),
            Instruction::Div(a, b) => write!(f, "{} / {}", a, b),
            Instruction::Mod(a, b) => write!(f, "{} % {}", a, b),
            Instruction::Eql(a, b) => write!(f, "{} ≓ {}", a, b),
        }
    }
}

struct ALU {
    input: Vec<isize>,
    register: [isize; 4],
}

impl ALU {
    fn new() -> ALU {
        ALU {
            input: vec![],
            register: [0; 4],
        }
    }

    fn run_program(&mut self, input: &str, instructions: &[Instruction]) {
        self.input = input.chars().map(|c| c.to_digit(10).unwrap() as isize).rev().collect_vec();
        instructions.iter().for_each(|i| self.execute(i));
    }


    fn execute(&mut self, i: &Instruction) {
        //println!("{:?}", i);
        match i {
            Instruction::Inp(a) => self.register[a.index()] = self.input.pop().unwrap(),
            Instruction::Add(a, b) => *self.var_mut(a) += self.deref(b),
            Instruction::Mul(a, b) => *self.var_mut(a) *= self.deref(b),
            Instruction::Div(a, b) => {
                debug_assert!(self.deref(b) != 0);
                *self.var_mut(a) /= self.deref(b)
            }
            Instruction::Mod(a, b) => {
                debug_assert!(*self.var_mut(a) >= 0 && self.deref(b) > 0);
                *self.var_mut(a) %= self.deref(b)
            }
            Instruction::Eql(a, b) => *self.var_mut(a) = self.eql(a, b),
        }
        //println!("{:?}", self.register);
    }

    fn var_mut(&mut self, p: &Pointer) -> &mut isize {
        &mut self.register[p.index()]
    }

    fn var(&self, p: &Pointer) -> &isize {
        &self.register[p.index()]
    }

    fn deref(&self, p: &Value) -> isize {
        match p {
            Value::Number(n) => *n,
            Value::Var(p) => *self.var(p)
        }
    }

    fn eql(&self, a: &Pointer, b: &Value) -> isize {
        if self.var(a) == &self.deref(b) { 1 } else { 0 }
    }
}


fn sub(z: isize, a: isize, i: isize) -> isize {
    if ((z % 26) + a) != i { 1 } else { 0 }
}

static params: [(isize, isize, isize); 14] = [
    (1, 12, 9),
    (1, 12, 4),
    (1, 12, 2),
    (26, -9, 5),
    (26, -9, 1),
    (1, 14, 6),
    (1, 14, 11),
    (26, -10, 15),
    (1, 15, 7),
    (26, -2, 12),
    (1, 11, 15),
    (26, -15, 9),
    (26, -9, 12),
    (26, -3, 12)
];

fn calc_z(i: &[isize], level: usize) -> isize {
    let (c, a, b) = params[level];
    let z = if level == 0 {
        0isize
    } else {
        calc_z(i, level - 1)
    };
    let i = i[level];
    ((z / c) * ((25 * sub(z, a, i)) + 1)) + ((i + b) * sub(z, a, i))
}

fn calc_level(z: isize, i: isize, level: isize) -> isize {
    let (c, a, b) = params[level as usize];
    ((z / c) * ((25 * sub(z, a, i)) + 1)) + ((i + b) * sub(z, a, i))
}

fn calc(i: &[isize]) -> isize {
    calc_z(i, 13)
}

fn formula() {
    let mut instructions = parse_input(read_lines("input/day_24.txt"));

    let mut count = -1;
    for chunk in instructions.chunks(18) {
        let mut formula = format!("{}", chunk.last().unwrap());
        count += 1;
        for i in chunk.iter().rev().skip(1) {
            let f = format!("({})", i);
            let pointer = match &i {
                Inp(a) => a,
                Add(a, _) => a,
                Mul(a, _) => a,
                Div(a, _) => a,
                Mod(a, _) => a,
                Eql(a, _) => a,
            };

            let replacement = match &i {
                Inp(_) => format!("i[{}]", count),
                Add(a, Number(0)) => format!("{}", a),
                Add(_, _) => f,
                Mul(_, Number(0)) => "0".to_string(),
                Mul(_, _) => f,
                Div(_, _) => f,
                Mod(_, _) => f,
                Eql(a, b) => format!("(if {}=={} {{1}} else {{0}})", a, b),
            };
            //println!("replacing {} with {}",&format!("{}", pointer), &f );
            formula = formula.replace(&format!("{}", pointer), &replacement);
            //println!("{}", formula);
        }


        println!("let z = {};", formula)
    }
}

fn print_term(instructions: &Vec<Instruction>) {
    instructions.iter().for_each(|i| {
        println!("{}", i)
    });
}


#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::{ALU, calc, calc_level, calc_z, Instruction, parse_input, Pointer, print_term, read_lines, sub};
    use crate::Instruction::*;
    use crate::Pointer::*;
    use crate::Value::{Number, Var};


    #[test]
    fn test_negate() {
        let instructions = vec![
            Inp(X),
            Mul(X, Number(-1)),
        ];
        let mut alu = ALU::new();
        alu.run_program("1", &instructions);
        assert_eq!(-1, *alu.var(&X));
    }

    #[test]
    fn test_three_times() {
        let instructions = vec![
            Inp(Z),
            Inp(X),
            Mul(Z, Number(3)),
            Eql(Z, Var(X)),
        ];
        let mut alu = ALU::new();
        alu.run_program("13", &instructions);
        assert_eq!(1, *alu.var(&Z));
        alu.run_program("14", &instructions);
        assert_eq!(0, *alu.var(&Z));
    }

    #[test]
    fn test_binary() {
        let instructions = vec![
            Inp(W),
            Add(Z, Var(W)),
            Mod(Z, Number(2)),
            Div(W, Number(2)),
            Add(Y, Var(W)),
            Mod(Y, Number(2)),
            Div(W, Number(2)),
            Add(X, Var(W)),
            Mod(X, Number(2)),
            Div(W, Number(2)),
            Mod(W, Number(2)),
        ];
        let mut alu = ALU::new();
        alu.run_program("0", &instructions);
        assert_eq!(0, *alu.var(&W));
        assert_eq!(0, *alu.var(&X));
        assert_eq!(0, *alu.var(&Y));
        assert_eq!(0, *alu.var(&Z));
        alu.run_program("9", &instructions);
        assert_eq!(1, *alu.var(&W));
        assert_eq!(0, *alu.var(&X));
        assert_eq!(0, *alu.var(&Y));
        assert_eq!(1, *alu.var(&Z));
    }

    #[test]
    fn test_model_number() {
        let instructions = parse_input(read_lines("input/day_24.txt"));
        assert_eq!(instructions.len(), 252);
        let mut alu = ALU::new();
        alu.run_program("13579246899999", &instructions);

        assert_eq!(0, *alu.var(&Pointer::Z));
    }

    #[test]
    fn analyze() {
        let l = read_lines("input/day_24.txt").collect_vec();
        (0..18).for_each(|i| {
            print!("{} ", l[i].split(' ').next().unwrap());
            (0..14).for_each(|j| {
                print!("{:<4}", l[i + j * 18].split_whitespace().skip(1).join(""))
            });
            println!()
        });
    }

    #[test]
    fn terms() {
        let instructions = parse_input(read_lines("input/day_24.txt"));

        print_term(&instructions)
    }

    #[test]
    fn formula() {
        let mut instructions = parse_input(read_lines("input/day_24.txt"));
        let mut formula = format!("{}", instructions.pop().unwrap());
        instructions.reverse();
        println!("start: {}", formula);

        let mut count = 0;
        for i in instructions.iter() {
            let f = format!("({})", i);
            let pointer = match &i {
                Inp(a) => a,
                Add(a, _) => a,
                Mul(a, _) => a,
                Div(a, _) => a,
                Mod(a, _) => a,
                Eql(a, _) => a,
            };
            //println!("replacing {} with {}",&format!("{}", pointer), &f );
            formula = formula.replace(&format!("{}", pointer), &f);
            //println!("{}", formula);
            if count > 150 {
                break;
            }
            count += 1
        }
        println!("{}", formula)
    }
}


fn find_iz_for_z(target_z: isize, level: isize) -> Vec<(isize, isize)> {
    let mut solutions = vec![];
    for i in 1..10 {
        let i = 10 - i;
        for z in -999..999 {
            let z2 = calc_level(z, i, level);
            if z2 == target_z {
                //println!("level: {}, i: {}, z: {} = z2 {}", level, i, z, z2);
                solutions.push((i, z));
            }
        }
    }
    solutions
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct State {
    target_z: isize,
    number: Vec<isize>,
    level: isize,
}

impl State {
    fn start() -> Self{
        Self {
            target_z: 0,
            number: vec![],
            level: 13
        }
    }

    fn successors(&self) -> Vec<(Self, isize)> {
        if self.level >= 0 {
            find_iz_for_z(self.target_z, self.level).into_iter()
                .map(|(n, target_z)| {
                    let mut number = self.number.clone();
                    number.push(n);
                    (Self {
                        target_z,
                        number,
                        level: self.level - 1
                    }, 10-n * (14-self.level))
                }).collect()
        } else {
            vec![]
        }
    }

    fn is_finished(&self) -> bool {
        self.number.len() == 14 && self.target_z == 0
    }

    fn heuristic(&self) -> isize {
        (14-self.number.len()) as isize
    }
}




fn find_star() -> String {
    let mut result = astar(&State::start(),
                       |state|  state.successors(),
                       |state| state.heuristic() ,
                       |s| s.is_finished())
        .map(|r|r.0.into_iter().collect_vec()).unwrap();

    println!("{:?}", &result);
    println!("{:?}", result.last().unwrap().number.iter().rev().collect_vec());

    todo!()
}



#[test]
fn experiment() {
    let mut i = vec![9, 9, 9, 9, 5, 6, 8, 9, 4, 9, 8, 9, 9, 9];
    let z = -3;
    let z = ((z / 1) * ((25 * (if (if ((z % 26) + 12)==i[0] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[0]) + 9) * (if (if ((z % 26) + 12)==i[0] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 1) * ((25 * (if (if ((z % 26) + 12)==i[1] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[1]) + 4) * (if (if ((z % 26) + 12)==i[1] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 1) * ((25 * (if (if ((z % 26) + 12)==i[2] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[2]) + 2) * (if (if ((z % 26) + 12)==i[2] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 26) * ((25 * (if (if ((z % 26) + -9)==i[3] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[3]) + 5) * (if (if ((z % 26) + -9)==i[3] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 26) * ((25 * (if (if ((z % 26) + -9)==i[4] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[4]) + 1) * (if (if ((z % 26) + -9)==i[4] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 1) * ((25 * (if (if ((z % 26) + 14)==i[5] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[5]) + 6) * (if (if ((z % 26) + 14)==i[5] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 1) * ((25 * (if (if ((z % 26) + 14)==i[6] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[6]) + 11) * (if (if ((z % 26) + 14)==i[6] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 26) * ((25 * (if (if ((z % 26) + -10)==i[7] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[7]) + 15) * (if (if ((z % 26) + -10)==i[7] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 1) * ((25 * (if (if ((z % 26) + 15)==i[8] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[8]) + 7) * (if (if ((z % 26) + 15)==i[8] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 26) * ((25 * (if (if ((z % 26) + -2)==i[9] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[9]) + 12) * (if (if ((z % 26) + -2)==i[9] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 1) * ((25 * (if (if ((z % 26) + 11)==i[10] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[10]) + 15) * (if (if ((z % 26) + 11)==i[10] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 26) * ((25 * (if (if ((z % 26) + -15)==i[11] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[11]) + 9) * (if (if ((z % 26) + -15)==i[11] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = ((z / 26) * ((25 * (if (if ((z % 26) + -9)==i[12] {1} else {0})==0 {1} else {0})) + 1)) + (((0 + i[12]) + 12) * (if (if ((z % 26) + -9)==i[12] {1} else {0})==0 {1} else {0}));
    println!("{}", z);
    let z = calc_level(z, i[13], 13);
    println!("{}", z);


    println!("----");
}

