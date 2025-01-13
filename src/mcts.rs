use crate::game::{Board, GameState, Player};

use deepsize::DeepSizeOf;
use rand::Rng;
use rayon::prelude::*;

#[derive(DeepSizeOf, Debug)]
pub(crate) struct MCTSArena {
    nodes: Vec<MCTSNode>,
}

#[derive(Copy, Clone, Debug, DeepSizeOf)]
pub struct NodeId(usize);

#[derive(Default, Debug, DeepSizeOf)]
pub struct MCTSNode {
    pub board: Board,
    pub wins: f32,
    pub visits: f32,
    // Node specific
    pub parent: Option<NodeId>,
    pub children: Option<Vec<NodeId>>,
}

enum BestNode {
    Expand(NodeId),
    NodeId(NodeId),
}

impl MCTSArena {
    pub fn init() -> Self {
        Self {
            nodes: vec![MCTSNode::default()],
        }
    }

    pub fn from(board: Board) -> Self {
        Self {
            nodes: vec![MCTSNode {
                board: board,
                wins: 0.0,
                visits: 0.0,
                parent: None,
                children: None,
            }],
        }
    }

    pub fn root(&self) -> NodeId {
        NodeId(0)
    }

    pub(crate) fn resolve(&self, id: &NodeId) -> &MCTSNode {
        &self.nodes[id.0]
    }

    fn resolve_mut(&mut self, id: &NodeId) -> &mut MCTSNode {
        &mut self.nodes[id.0]
    }

    pub fn analyze(&mut self, id: NodeId, mut n_iters: u32) -> (f32, NodeId) {
        let mut simulation_results = Vec::new();
        while n_iters > 0 {
            match self.select(id, 2.0f32.sqrt()) {
                BestNode::Expand(to_expand_id) => {
                    self.expand(to_expand_id);
                    let expanded_node = self.resolve(&to_expand_id);
                    // The vector is cleared before collecting
                    expanded_node
                        .children
                        .as_ref()
                        .expect("Non terminal node can't have 0 children")
                        .par_iter()
                        .map(|child_id| (*child_id, self.simulate(child_id)))
                        .collect_into_vec(&mut simulation_results);
                }
                BestNode::NodeId(terminal_node_id) => {
                    let terminal_node = self.resolve(&terminal_node_id);
                    let result = terminal_node.board.check_game_state();
                    simulation_results.push((terminal_node_id, result));
                }
            }
            let player = self.resolve(&id).board.next_player;
            self.backpropagate(&simulation_results, &player);
            n_iters -= 1;
        }

        let best_child_id = self.select_best_child(id);
        let best_child = self.resolve(&best_child_id);
        (best_child.wins / best_child.visits * 100.0, best_child_id)
    }

    fn select_best_child(&self, mut id: NodeId) -> NodeId {
        let node = self.resolve(&id);
        let children = node.children.as_ref().expect("Node is terminal");
        let mut max_uct = 0.0;
        let mut max_uct_index = 0;
        for i in 0..children.len() {
            let child = self.resolve(&children[i]);
            let uct = child.visits;
            if uct > max_uct {
                max_uct = uct;
                max_uct_index = i;
            }
            id = children[max_uct_index];
        }

        id
    }

    fn select(&self, mut id: NodeId, c: f32) -> BestNode {
        let mut node = self.resolve(&id);
        while !node.board.game_over() {
            match &node.children {
                None => {
                    return BestNode::Expand(id);
                }
                Some(children) => {
                    let mut max_uct = 0.0;
                    let mut max_uct_index = 0;
                    for i in 0..children.len() {
                        let child = self.resolve(&children[i]);
                        let uct = child.wins / child.visits
                            + c * (node.visits.ln() / child.visits).sqrt();
                        if uct > max_uct {
                            max_uct = uct;
                            max_uct_index = i;
                        }
                        id = children[max_uct_index];
                    }
                    node = self.resolve(&id);
                }
            }
        }
        BestNode::NodeId(id)
    }

    fn expand(&mut self, id: NodeId) {
        let node = self.resolve(&id);
        let moves = node.board.get_moves();

        let mut children = vec![];
        // TODO: Optimize
        for i in 0..81 {
            if (moves >> i) & 1 == 1 {
                let node = self.resolve_mut(&id);
                let board = node.board.unchecked_play(Board::move_from_index(i));
                let child_node = MCTSNode {
                    board,
                    wins: 0.0,
                    visits: 0.0,
                    parent: Some(id),
                    children: None,
                };
                self.nodes.push(child_node);
                children.push(NodeId(self.nodes.len() - 1));
            }
        }
        let node = self.resolve_mut(&id);
        node.children = Some(children);
    }

    fn simulate(&self, id: &NodeId) -> GameState {
        let node = self.resolve(id);

        let mut board = node.board.clone();

        // TODO: Repeats check 2 times when game is over. Make it 1.
        while !board.game_over() {
            let moves = board.get_moves();
            let num_moves = moves.count_ones();

            let random_move_number = rand::thread_rng().gen_range(0..num_moves);
            let move_index =
                find_kth_high_bit_index(moves, random_move_number).expect("Precalculated");
            board = board.unchecked_play(Board::move_from_index(move_index));
        }

        board.check_game_state()
    }

    fn backpropagate(&mut self, simulation_results: &Vec<(NodeId, GameState)>, player: &Player) {
        for (id, result) in simulation_results {
            match result {
                GameState::InProgress => unreachable!(),
                GameState::Won(winner) => {
                    let mut node = self.resolve_mut(id);
                    loop {
                        node.visits += 1.0;
                        if player == winner {
                            node.wins += 1.0;
                        }
                        if let Some(parent) = node.parent {
                            node = self.resolve_mut(&parent);
                        } else {
                            break;
                        }
                    }
                }
                GameState::Draw => {
                    let mut node = self.resolve_mut(id);
                    loop {
                        node.wins += 1e-8;
                        node.visits += 1.0;
                        if let Some(parent) = node.parent {
                            node = self.resolve_mut(&parent);
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn find_kth_high_bit_index(n: u128, k: u32) -> Option<u8> {
    let mut count = 0;

    for i in 0..81 {
        if n & (1 << i) != 0 {
            if count == k {
                return Some(i);
            }
            count += 1;
        }
    }

    None
}
