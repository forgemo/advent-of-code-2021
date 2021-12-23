mod lib;

use std::usize;
use itertools::{Itertools,};
use crate::Body::Literal;
use crate::lib::{read_lines};

fn main() {
    let (result_a, result_b) = task(read_lines("input/day_16.txt"));
    assert_eq!(result_a, 955);
    assert_eq!(result_b, 158135423448);

    println!("task-a: {}, task-b: {}", result_a, result_b);
}

fn task(lines: impl Iterator<Item=String>) -> (usize, usize) {
   let hex = read_input(lines);
    let mut reader = BitsReader::from_string(&hex);
    let packet = reader.read_packet();
    (packet.version_sum(), packet.value())
}

struct BitsReader<'a> {
    bits: Box<dyn Iterator<Item=usize> + 'a>,
    read_bits_count: usize
}

impl <'a>BitsReader<'a> {

    fn from_string(hex: &'a String) -> BitsReader {
        Self::from_str(hex.as_str())
    }

    fn from_str(hex: &str) -> BitsReader {
        let bits = Box::new(hex.chars()
            .map(|c|hex_to_binary(&c))
            .flatten());
        BitsReader {bits, read_bits_count: 0}
    }

    fn read_packet(&mut self) -> Packet {
        let header = self.read_header();
        let content = match header.type_id {
            0 => Body::Operator(Op::Sum, self.read_operator_body()),
            1 => Body::Operator(Op::Prod,self.read_operator_body()),
            2 => Body::Operator(Op::Min,self.read_operator_body()),
            3 => Body::Operator(Op::Max,self.read_operator_body()),
            4 => Body::Literal(self.read_literal_value()),
            5 => Body::Operator(Op::Gt,self.read_operator_body()),
            6 => Body::Operator(Op::Lt,self.read_operator_body()),
            7 => Body::Operator(Op::Eq,self.read_operator_body()),
            _ => panic!("unexpected type id")
        };
        Packet{ header, body: content }
    }

    fn read_literal_value(&mut self) -> usize {
        let mut last = false;
        let mut i = -1;
        let mut literal_bits = vec![];
        while !last || i < 4 {
            i = if i == 4 {0} else { i + 1};
            if !last || i <= 4 {
                let bit = self.bits.next().expect("reading bit failed");
                self.read_bits_count += 1;
                last = last || (i == 0 && bit==0);
                if i != 0 {
                    literal_bits.push(bit);
                }
            }
        }
        parse_bin(&literal_bits.iter().join(""))
    }

    fn read_header(&mut self) -> Header {
        let version = self.bits.by_ref().take(3).join("");
        let type_id = self.bits.by_ref().take(3).join("");
        self.read_bits_count += 6;
        Header{
            version: parse_bin(&version),
            type_id: parse_bin(&type_id)
        }
    }

    fn read_length_type_id(&mut self) -> usize {
        let id = self.bits.by_ref().next().expect("parse failed");
        self.read_bits_count += 1;
        id
    }

    fn read_length_type_0_body(&mut self) -> Vec<Packet> {
        let length = parse_bin(&self.bits.by_ref().take(15).join(""));
        self.read_bits_count += 15;
        let mut packets = vec![];
        let start_count = self.read_bits_count;
        while self.read_bits_count - start_count < length {
            packets.push(self.read_packet());
        }
        packets
    }

    fn read_length_type_1_body(&mut self) -> Vec<Packet> {
        let count = parse_bin(&self.bits.by_ref().take(11).join(""));
        self.read_bits_count += 11;
        (0..count).map(|_|self.read_packet()).collect_vec()
    }

    fn read_operator_body(&mut self) -> Vec<Packet> {
        let length_type= self.read_length_type_id();
        match length_type {
            0 => self.read_length_type_0_body(),
            1 => self.read_length_type_1_body(),
            _ => panic!("unexpected length type")
        }
    }
}


fn hex_to_binary(hex: &char) -> [usize;4] {
    match hex {
        '0' => [0,0,0,0],
        '1' => [0,0,0,1],
        '2' => [0,0,1,0],
        '3' => [0,0,1,1],
        '4' => [0,1,0,0],
        '5' => [0,1,0,1],
        '6' => [0,1,1,0],
        '7' => [0,1,1,1],
        '8' => [1,0,0,0],
        '9' => [1,0,0,1],
        'A' => [1,0,1,0],
        'B' => [1,0,1,1],
        'C' => [1,1,0,0],
        'D' => [1,1,0,1],
        'E' => [1,1,1,0],
        'F' => [1,1,1,1],
        _ => panic!("illegal char")
    }
}

impl Packet {
    fn version_sum(&self) -> usize {
        self.header.version + match &self.body {
             Literal(_) => 0,
             Body::Operator(_, v) => v.iter().map(|p|p.version_sum()).sum()
        }
    }

    fn value(&self) -> usize {
        match &self.body {
            Literal(v) => *v,
            Body::Operator(o, v) => match o {
                Op::Min => v.iter().map(|p|p.value()).min().unwrap(),
                Op::Max => v.iter().map(|p|p.value()).max().unwrap(),
                Op::Sum => v.iter().map(|p|p.value()).sum(),
                Op::Prod => v.iter().map(|p|p.value()).product(),
                Op::Gt => if v[0].value() > v[1].value() {1} else {0},
                Op::Lt => if v[0].value() < v[1].value() {1} else {0},
                Op::Eq => if v[0].value() == v[1].value() {1} else {0},
            }
        }
    }
}
#[derive(Debug, PartialEq)]
struct Packet {
    header: Header,
    body: Body
}

#[derive(Debug, PartialEq)]
enum Op {Sum, Prod, Min, Max, Gt, Lt, Eq}

#[derive(Debug, PartialEq)]
enum Body {
    Literal(usize), Operator(Op, Vec<Packet>)
}

#[derive(Debug, PartialEq)]
struct Header {
    version: usize,
    type_id: usize
}

fn read_input(mut lines: impl Iterator<Item=String>) -> String {
    lines.next().expect("empty file")
}

fn parse_bin(s: &String) -> usize {
    usize::from_str_radix(s, 2).expect("parse failed")
}