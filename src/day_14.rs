mod lib;

use std::collections::HashMap;
use itertools::{Itertools, MinMaxResult};
use rayon::prelude::*;
use crate::lib::{read_lines};

fn main() {
    let result_a = task(read_lines("input/day_14.txt"), 10);
    assert_eq!(result_a, 2621);

    let result_b = task(read_lines("input/day_14.txt"), 40);
    assert_eq!(result_b, 2843834241366);

    println!("task-a: {}, task-b: {}", result_a, result_b);
}

fn task(lines: impl Iterator<Item=String>, steps: usize) -> usize {
    let (template, rules) = read_input(lines);
    let rule_map = rules.iter().map(|r|(r.0.clone(), r.clone())).collect::<HashMap<_,_>>();
    let seed = template.iter().tuple_windows().collect_vec().into_iter().map(|(a, b)|[*a, *b]);
    let mut stats = seed.collect_vec().par_iter()
         .map(|seed| {
             recursive_edit(seed, steps,&rule_map,&mut Cache::new())
         }).reduce_with(|a: Stats, b: Stats| merge_stats(&a,&b))
        .expect("reduce failed");

    *stats.entry(template[template.len()-1]).or_insert(0) += 1;
    score(&stats)
}

fn single_edit(pair: &[char; 2], rules: &Rules) -> Vec<Element>  {
    match rules.get(pair) {
        None => vec![pair.clone()],
        Some((_, insert)) => vec![[pair[0], *insert],[*insert, pair[1]]],
    }
}

fn recursive_edit(
    element: &Element,
    step: usize,
    rules: &Rules,
    cache: &mut Cache) -> Stats {

    let key = cache_key(element, &step);
    if let Some(cached) = cache.get(&key){
        cached.clone()
    } else {
        let stats = if step - 1 > 0  {
            let edit = single_edit(&element, rules);
            edit.iter()
                .map(|e|recursive_edit(e, step-1, rules, cache))
                .reduce(|a,b|merge_stats(&a, &b))
                .expect("reduce failed")
        } else {
            let edit = single_edit(&element, rules);
            stats(&edit)
        };

        cache.insert(key, stats.clone());
        stats
    }
}


fn stats(elements: &Vec<Element>) -> Stats {
    elements.iter().map(|pair|pair[0])
        .fold(Stats::new(), |mut stat,c| {
            *stat.entry(c).or_insert(0) += 1;
            stat
        })
}

fn merge_stats(a: &Stats, b: &Stats) -> Stats {
    let mut m = a.clone();
    b.iter().for_each(|(k,v)| *m.entry(*k).or_insert(0) += v);
    m
}

fn cache_key(element: &Element, step: &usize) -> String {
    format!("{}{}#{}", element[0], element[1], step)
}


fn score(stat: &HashMap<char,usize>) -> usize {
    let  result= stat.iter().minmax_by_key(|(_,count)|**count);
    match result {
        MinMaxResult::MinMax((_ , min), (_, max)) => max-min,
        _ => panic!("unexpected result")
    }
}

type Rule = ([char;2], char);
type Rules = HashMap<[char;2], Rule>;
type Element = [char;2];
type Stats = HashMap<char, usize>;
type Cache = HashMap<String, Stats>;


fn read_input(mut lines: impl Iterator<Item=String>) -> (Vec<char>, Vec<Rule>) {
    let template = lines.next().expect("invalid input").chars().collect_vec();
    let rules = lines.skip(1).map(|s| s
        .split(" -> ")
        .map(|s|s.chars().collect_vec())
        .collect_tuple()
        .map(|(p, i)| ([p[0], p[1]], i[0]))
        .expect("invalid input")
    ).collect_vec();
    (template, rules)
}