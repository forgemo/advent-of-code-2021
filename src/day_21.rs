use std::collections::HashMap;
use std::usize;
use itertools::{Itertools};

mod lib;

fn main() {
    let task_a = task_a();
    let task_b = task_b();
    assert_eq!(598416, task_a);
    assert_eq!(27674034218179, task_b);
    println!("task_a: {}, task_b: {}", task_a, task_b);
}

fn task_a() -> usize {
    let mut game = Game::new_with( vec![
            Player::new(1),
            Player::new(2)
        ],
       1000,
       deterministic_die(100)
    );
    game.run_until_win()
}

fn task_b() -> usize {
    let game = Game::new_with( vec![
            Player::new(1),
            Player::new(2)
        ],
       21,
       deterministic_die(3)
    );
    *game.count_universal_wins(&mut HashMap::new()).iter().max().unwrap()
}

#[derive(Clone)]
struct Player {
    score: usize,
    spaces: CyclingIterator
}

#[derive(Clone)]
struct Game {
    target_score: usize,
    players: Vec<Player>,
    die: CyclingIterator,
    turns: CyclingIterator,
    actions: CyclingIterator,
}

#[derive(Clone)]
struct CyclingIterator {
    min: usize,
    max: usize,
    state: usize,
    count: usize,
}


impl CyclingIterator {
    fn new(min: usize, max: usize, starting_at: usize) -> Self {
        debug_assert!(min < max);
        debug_assert!(starting_at >= min && starting_at <= max);
        let state = starting_at.checked_sub(1).unwrap_or(max);
        CyclingIterator {
            min, max, state, count: 0
        }
    }
}
impl Iterator for CyclingIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.state += 1;
        self.count += 1;
        if self.state > self.max {
            self.state = self.min;
        }
        Some(self.state)
    }
}

impl Player {
    fn new(starting_at: usize) -> Self {
        let spaces = CyclingIterator::new(1, 10, starting_at+1);
        Player {score: 0, spaces}
    }
    fn move_forward(&mut self, steps: usize) -> usize {
        self.spaces.by_ref().nth(steps-1).unwrap()
    }
}

impl Game {
    fn new_with(players: Vec<Player>, target_score: usize,  die: CyclingIterator) -> Self {
        let turns = CyclingIterator::new(0, 1, 0);
        let actions = CyclingIterator::new(0, 2, 0);
        Self { players, die, turns, target_score , actions }
    }

    fn roll_die(&mut self) -> usize {
        self.die.by_ref().next().unwrap()
    }

    fn next_roll(&mut self, roll: usize) -> bool {
        let action = self.actions.next().unwrap();
        if action == 0 {
            self.turns.next().unwrap();
        }
        let player = &mut self.players[self.turns.state];
        let value = player.move_forward(roll);
        if action == 2 {
            player.score += value;
        }

        action == 2 && player.score >= self.target_score
    }

    fn count_universal_wins(&self, cache: &mut HashMap<String, Vec<usize>>) -> Vec<usize> {
        (1..4).map(|roll| {
            let mut game = self.clone();
            let is_over = game.next_roll(roll);
            if is_over {
                let mut stats = vec![0;2];
                stats[game.turns.state] += 1;
                stats
            } else {
                let key = game.cache_key();
                if let Some(cached) = cache.get(&key) {
                    cached.clone()
                } else {
                    let stats = game.count_universal_wins(cache);
                    cache.insert(key, stats.clone());
                    stats
                }
            }
        }).reduce(|mut accum, stats| {
            accum[0] += stats[0];
            accum[1] += stats[1];
            accum
        }).unwrap()
    }

    fn run_until_win(&mut self) -> usize {
        loop {
            let roll = self.roll_die();
            if self.next_roll(roll) {
               return self.score_of_loosing_player() * self.die.count;
            }
        }
    }

    fn score_of_loosing_player(&self) -> usize {
        self.players.iter().map(|p|p.score).min().unwrap()
    }

    fn cache_key(&self) -> String {
        vec![self.turns.state,
             self.die.state,
             self.players[0].spaces.state,
             self.players[1].spaces.state,
             self.players[0].score,
             self.players[1].score,
             self.actions.state
        ].iter().join("")
    }

}


fn deterministic_die(sides: usize) -> CyclingIterator {
    CyclingIterator::new(1, sides, 1)
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use itertools::Itertools;
    use crate::{deterministic_die, Game, Player};

    #[test]
    fn test_example_0() {

        let rolls = deterministic_die(6).take(7).collect_vec();
        assert_eq!(vec![1, 2, 3, 4, 5, 6, 1], rolls);
        let r = deterministic_die(6).nth(2).unwrap();
        assert_eq!(3, r);
        let r = deterministic_die(6).nth(7).unwrap();
        assert_eq!(2, r);

        let mut p = Player::new(4);
        let s = p.move_forward(3);
        assert_eq!(p.spaces.state, 7);
        let s = p.move_forward(3);
        assert_eq!(p.spaces.state, 10);

        let players = vec![Player::new(4), Player::new(8)];
        let mut game = Game::new_with(players, 1000, deterministic_die(100));
        let points = game.run_until_win();


        assert_eq!(1000, game.players[0].score);
        assert_eq!(993, game.die.count);
        assert_eq!(739785, points);


        let players = vec![Player::new(4), Player::new(8)];
        let mut game = Game::new_with(players, 21, deterministic_die(3));
        println!("{:?}", game.count_universal_wins(&mut HashMap::new()));

    }

}

