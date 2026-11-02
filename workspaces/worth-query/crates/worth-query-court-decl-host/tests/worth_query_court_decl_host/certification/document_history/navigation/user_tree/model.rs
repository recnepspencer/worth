//! Independent logical reference: no SDK types, issued IDs or metadata reads.
#[derive(Clone, Copy, Debug)]
pub(super) enum Command {
    Edit(u64),
    Undo,
    Redo(usize),
    Checkout(usize),
}

pub(super) struct Node {
    pub(super) parent: Option<usize>,
    pub(super) value: u64,
}

pub(super) struct UserTree {
    pub(super) nodes: Vec<Node>,
    pub(super) selected: usize,
}

impl UserTree {
    pub(super) fn new(value: u64) -> Self {
        Self {
            nodes: vec![Node {
                parent: None,
                value,
            }],
            selected: 0,
        }
    }

    pub(super) fn children(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| (node.parent == Some(self.selected)).then_some(index))
            .collect()
    }

    pub(super) fn apply(&mut self, command: Command) {
        match command {
            Command::Edit(value) => {
                let parent = self.selected;
                self.selected = self.nodes.len();
                self.nodes.push(Node {
                    parent: Some(parent),
                    value,
                });
            }
            Command::Undo => {
                self.selected = self.nodes[self.selected].parent.expect("generated parent")
            }
            Command::Redo(child) => {
                assert_eq!(self.nodes[child].parent, Some(self.selected));
                self.selected = child;
            }
            Command::Checkout(node) => self.selected = node,
        }
    }

    pub(super) fn generated(&self, index: usize, seed: u64) -> Command {
        match index % 4 {
            0 => {
                let mut value = 3 + seed % 6;
                if value == self.nodes[self.selected].value {
                    value = 3 + (value - 2) % 6;
                }
                Command::Edit(value)
            }
            1 => Command::Undo,
            2 => {
                let children = self.children();
                Command::Redo(children[seed as usize % children.len()])
            }
            _ => {
                let mut node = seed as usize % self.nodes.len();
                if node == self.selected {
                    node = (node + 1) % self.nodes.len();
                }
                Command::Checkout(node)
            }
        }
    }
}
