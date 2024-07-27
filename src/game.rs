use std::simd::{cmp::SimdPartialEq, u16x8};

use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, deepsize::DeepSizeOf)]
pub enum Player {
    X,
    O,
}

#[derive(Debug)]
pub enum GameState {
    Won(Player),
    Draw,
    InProgress,
}

impl Default for Player {
    fn default() -> Self {
        Self::X
    }
}

impl Player {
    pub fn other(&self) -> Self {
        match self {
            Player::O => Player::X,
            Player::X => Player::O,
        }
    }
}

#[derive(Clone, Copy, Default, Debug, deepsize::DeepSizeOf)]
pub struct Board {
    pub x: u128,
    pub o: u128,
    pub gx: u16,
    pub go: u16,
    pub next_player: Player,
    pub last_move: Option<u8>,
}

const WIN_MASKS: u16x8 = u16x8::from_array([
    // Horizontal
    0b111_000_000,
    0b000_111_000,
    0b000_000_111,
    // vertical
    0b100_100_100,
    0b010_010_010,
    0b001_001_001,
    // Diagonal
    0b100_010_001,
    0b001_010_100,
]);

impl Board {
    pub fn move_from_gl(global: u8, local: u8) -> u8 {
        (global << 4) | local
    }

    pub fn move_from_index(index: u8) -> u8 {
        let global = index / 9;
        let local = index % 9;
        Self::move_from_gl(global, local)
    }

    pub fn game_over(&self) -> bool {
        match self.check_game_state() {
            GameState::Won(_) | GameState::Draw => true,
            GameState::InProgress => false,
        }
    }

    fn update_board_state(&mut self, global: u8) -> GameState {
        let xbits = (self.x >> global * 9) & 0b111_111_111;
        let obits = (self.o >> global * 9) & 0b111_111_111;
        // let board = match self.next_player {
        //     Player::X => xbits,
        //     Player::O => obits,
        // };

        // match (u16x8::splat(board as u16) & WIN_MASKS)
        //     .simd_eq(WIN_MASKS)
        //     .any()
        // {
        //     true => {
        //         match self.next_player {
        //             Player::X => self.gx |= 1 << global,
        //             Player::O => self.go |= 1 << global,
        //         };
        //         GameState::Won(self.next_player)
        //     }
        //     false => {
        //         if xbits | obits == 0b111_111_111 {
        //             self.gx |= 1 << global;
        //             self.go |= 1 << global;
        //             GameState::Draw
        //         } else {
        //             GameState::InProgress
        //         }
        //     }
        // }
        if (u16x8::splat(xbits as u16) & WIN_MASKS)
            .simd_eq(WIN_MASKS)
            .any()
        {
            self.gx |= 1 << global;
            GameState::Won(Player::X)
        } else if (u16x8::splat(obits as u16) & WIN_MASKS)
            .simd_eq(WIN_MASKS)
            .any()
        {
            self.go |= 1 << global;
            GameState::Won(Player::O)
        } else {
            if xbits | obits == 0b111_111_111 {
                self.gx |= 1 << global;
                self.go |= 1 << global;
                GameState::Draw
            } else {
                GameState::InProgress
            }
        }
    }

    fn check_board_state(&self, global: u8) -> GameState {
        let xbits = (self.x >> global * 9) & 0b111_111_111;
        let obits = (self.o >> global * 9) & 0b111_111_111;

        if (u16x8::splat(xbits as u16) & WIN_MASKS)
            .simd_eq(WIN_MASKS)
            .any()
        {
            GameState::Won(Player::X)
        } else if (u16x8::splat(obits as u16) & WIN_MASKS)
            .simd_eq(WIN_MASKS)
            .any()
        {
            GameState::Won(Player::O)
        } else {
            if xbits | obits == 0b111_111_111 {
                GameState::Draw
            } else {
                GameState::InProgress
            }
        }
    }

    pub fn check_game_state(&self) -> GameState {
        let drawn_boards = self.gx & self.go;
        if (u16x8::splat(self.gx & !drawn_boards) & WIN_MASKS)
            .simd_eq(WIN_MASKS)
            .any()
        {
            GameState::Won(Player::X)
        } else if (u16x8::splat(self.go & !drawn_boards) & WIN_MASKS)
            .simd_eq(WIN_MASKS)
            .any()
        {
            GameState::Won(Player::O)
        } else if self.gx | self.go == 0b111_111_111 {
            GameState::Draw
        } else {
            GameState::InProgress
        }
    }

    /// Does not check validity of the moves
    pub fn unchecked_play(&self, m: u8) -> Self {
        let mut board = self.clone();

        let local = m & 0b1111;
        let global = (m >> 4) & 0b1111;

        match self.next_player {
            Player::X => board.x = self.x | ((1 << (global * 9)) << local),
            Player::O => board.o = self.o | ((1 << (global * 9)) << local),
        }
        board.update_board_state(global);
        board.last_move = Some(m);
        board.next_player = self.next_player.other();

        board
    }

    pub fn global_board_mask(&self) -> u128 {
        let mut mask = 0;
        for i in 0..9 {
            if (self.gx | self.go) & (1 << (8 - i)) != 0 {
                mask <<= 9;
                mask |= 0b111_111_111;
            } else {
                mask <<= 9;
            }
        }
        mask
    }

    pub fn get_moves(&self) -> u128 {
        match self.last_move {
            None => 0x1ffffffffffffffffffff,
            Some(m) => {
                let local = m & 0b1111;

                match self.check_board_state(local) {
                    GameState::Won(_) | GameState::Draw => {
                        !(self.x | self.o | self.global_board_mask()) & 0x1ffffffffffffffffffff
                    }
                    GameState::InProgress => {
                        !(self.x | self.o) & 0x1ffffffffffffffffffff & (0b111_111_111 << 9 * local)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod board_tests {
    use crate::game::Board;

    // #[test]
    fn test_valid_moves() {
        let board = Board::default();

        assert_eq!(board.get_moves(), 0x1ffffffffffffffffffff);
        let board = board.unchecked_play(Board::move_from_gl(0, 0));

        assert_eq!(board.get_moves(), 0b111_111_110);
        let board = board.unchecked_play(Board::move_from_gl(0, 4));

        assert_eq!(board.get_moves(), 0x1ff000000000);
    }
}
