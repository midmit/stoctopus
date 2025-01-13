#![feature(portable_simd)]

use std::fmt::Display;

use deepsize::DeepSizeOf;
use game::GameState;
use mcts::{MCTSArena, MCTSNode, NodeId};

mod game;
mod mcts;

pub struct Engine {
    arena: mcts::MCTSArena,
    current_node: NodeId,
}

#[derive(Debug)]
pub struct Evaluation {
    pub confidence: f32,
    pub best_move: NodeId,
}

#[derive(Debug)]
pub enum Error {
    IllegalMove,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalMove => f.write_str("Illegal move"),
        }
    }
}

impl std::error::Error for Error {}

impl Engine {
    pub fn init() -> Self {
        let arena = MCTSArena::init();

        Self {
            current_node: arena.root(),
            arena,
        }
    }

    pub fn analyze(&mut self, n_iters: u32) -> Evaluation {
        self.arena = MCTSArena::from(self.arena.resolve(&self.current_node).board);
        let (confidence, best_node) = self.arena.analyze(self.arena.root(), n_iters);

        return Evaluation {
            confidence,
            best_move: best_node,
        };
    }

    pub fn step(&mut self, r#move: NodeId) {
        self.current_node = r#move;
    }

    pub fn play(&mut self, mve: (u8, u8)) -> Result<(), Error> {
        let node = self.arena.resolve(&self.current_node);
        if let Some(children) = &node.children {
            for child in children {
                let child_node = self.arena.resolve(child);
                if let Some(last_move) = child_node.board.last_move {
                    if last_move == ((mve.0 << 4) | mve.1) {
                        return Ok(self.step(*child));
                    }
                }
            }
            return Err(Error::IllegalMove);
        } else {
            self.arena.analyze(self.current_node, 1);
            return self.play(mve);
        }
    }

    pub fn print_board(&self) {
        let board = self.arena.resolve(&self.current_node).board;

        for i in 0..3 {
            for j in 0..3 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 2 {
                print!("|");
            }
        }
        println!();

        for i in 0..3 {
            for j in 3..6 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 2 {
                print!("|");
            }
        }
        println!();

        for i in 0..3 {
            for j in 6..9 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 2 {
                print!("|");
            }
        }
        println!();
        println!("---------+---------+---------");
        for i in 3..6 {
            for j in 0..3 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 5 {
                print!("|");
            }
        }
        println!();

        for i in 3..6 {
            for j in 3..6 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 5 {
                print!("|");
            }
        }
        println!();

        for i in 3..6 {
            for j in 6..9 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 5 {
                print!("|");
            }
        }
        println!();
        println!("---------+---------+---------");
        for i in 6..9 {
            for j in 0..3 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 8 {
                print!("|");
            }
        }
        println!();

        for i in 6..9 {
            for j in 3..6 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 8 {
                print!("|");
            }
        }
        println!();

        for i in 6..9 {
            for j in 6..9 {
                if board.x & (1 << (i * 9 + j)) != 0 {
                    print!(" X ");
                } else if board.o & (1 << (i * 9 + j)) != 0 {
                    print!(" O ");
                } else {
                    print!("   ");
                }
            }
            if i != 8 {
                print!("|");
            }
        }
        println!();
    }

    pub fn is_game_over(&self) -> bool {
        let node = self.arena.resolve(&self.current_node);
        node.board.game_over()
    }
    pub fn game_state(&self) -> GameState {
        let node = self.arena.resolve(&self.current_node);
        node.board.check_game_state()
    }

    pub fn memory(&self) -> usize {
        self.arena.deep_size_of()
    }

    pub fn resolve_node(&self, id: &NodeId) -> &MCTSNode {
        self.arena.resolve(id)
    }
}

#[cfg(test)]
mod engine_tests {
    use crate::Engine;

    #[test]
    fn test_engine() {
        let mut engine = Engine::init();
        let mut move_count = 0;
        while !engine.is_game_over() {
            move_count += 1;
            println!();
            engine.print_board();
            let ev = engine.analyze(5000);
            let node = engine.arena.resolve(&ev.best_move);
            println!(
                "\nConfidence {}%, Best Move: {},{}",
                ev.confidence,
                (node.board.last_move.unwrap() >> 4) & 0b1111,
                node.board.last_move.unwrap() & 0b1111
            );
            engine.step(ev.best_move);
        }

        println!("\n-----------------------------\n");

        engine.print_board();
        println!(
            "Result: {:?}; Move Count: {move_count}",
            engine.game_state()
        );
    }

    // #[test]
    fn test_play() {
        let mut engine = Engine::init();
        engine.play((4, 4)).unwrap();
        engine.play((4, 0)).unwrap();
        println!(
            "{:?} \nMemory: {}mb",
            engine.analyze(1000),
            engine.memory() as f32 / 1000.0 / 1000.0
        );
    }
}
