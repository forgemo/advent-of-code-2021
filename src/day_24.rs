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
    let task_a = task_a(read_lines("input/day_24.txt"));
    assert_eq!(580810, task_a);
    let task_b = task_b(read_lines("input/day_24.txt"));
    assert_eq!(1265621119006734, task_b);
    println!("task_a: {}, task_b: {}", task_a, task_b);
}


fn task_a(lines: impl Iterator<Item=String>) -> isize {
    let instructions = parse_input(lines);
    let max = (10000000000000usize .. 99999999999999+1)
        .into_par_iter()
        .map(|i| (99999999999999-i).to_string() )
        .map(|n| {
            let mut alu = ALU::new();
            alu.run_program(n.as_str(), &instructions);
            *alu.var(&Pointer::Z)
        })
        .filter(|z| {
            if z == &0 {
                println!("found {}", &z);
                true
            } else {
                false
            }
        })
        .max().unwrap();

    println!("max: {}", max);
    max
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
    Var(Pointer)
}

#[derive(Clone, Debug)]
enum Pointer {
    W, X, Y, Z
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


#[derive(Clone, Debug)]
enum Instruction {
    Inp(Pointer),
    Add(Pointer, Value),
    Mul(Pointer, Value),
    Div(Pointer, Value),
    Mod(Pointer, Value),
    Eql(Pointer, Value),
}

struct ALU {
    input: Vec<isize>,
    register: [isize;4]
}

impl ALU {

    fn new() -> ALU {
        ALU {
            input: vec![],
            register: [0;4]
        }
    }

    fn run_program(&mut self, input: &str, instructions: &[Instruction]) {
        self.input = input.chars().map(|c| c.to_digit(10).unwrap() as isize).rev().collect_vec();
        instructions.iter().for_each(|i|self.execute(i));
    }


    fn execute(&mut self, i: &Instruction)  {
        //println!("{:?}", i);
        match i {
            Instruction::Inp(a) => self.register[a.index()] = self.input.pop().unwrap(),
            Instruction::Add(a, b) => *self.var_mut(a) += self.deref(b),
            Instruction::Mul(a, b) => *self.var_mut(a) *= self.deref(b),
            Instruction::Div(a, b) => {
                debug_assert!(self.deref(b) != 0);
                *self.var_mut(a) /= self.deref(b)
            },
            Instruction::Mod(a, b) => {
                debug_assert!(*self.var_mut(a) >= 0 && self.deref(b) > 0);
                *self.var_mut(a) %= self.deref(b)
            },
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
        if self.var(a) == &self.deref(b) {1} else {0}
    }


}


#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::{ALU, Instruction, parse_input, Pointer, read_lines};
    use crate::Instruction::*;
    use crate::Pointer::*;
    use crate::Value::{Number, Var};


    #[test]
    fn test_negate() {
        let instructions = vec![
            Inp(X),
            Mul(X, Number(-1))
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
            Eql(Z, Var(X))
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
                print!("{:<4}", l[i+j*18].split_whitespace().skip(1).join(""))
            });
            println!()
        });
    }

}

