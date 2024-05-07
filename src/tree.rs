#[derive(Default, Copy, Clone)]
pub struct State {
    sum: f32,
    max: f32,
    min: f32,
    count: u64,
}

#[derive(Clone, Copy)]
pub struct Node {
    name: [u8; 32],
    state: State,
    l_child: Option<usize>,
    r_child: Option<usize>,
}

impl Node {
    pub fn new(name: [u8; 32], value: f32) -> Self {
        Self {
            name,
            state: State {
                count: 1,
                sum: value,
                min: value,
                max: value,
            },
            l_child: None,
            r_child: None,
        }
    }

    pub fn update_state(&mut self, num: f32) {
        self.state.count += 1;
        self.state.sum += num;
        self.state.max = self.state.max.max(num);
        self.state.min = self.state.min.min(num);
    }
}

pub struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn update(&mut self, name: [u8; 32], value: f32) {
        if self.nodes.is_empty() {
            self.nodes.push(Node::new(name, value));
        } else {
            self.update_node(0, name, value);
        }
    }

    fn update_node(&mut self, idx: usize, name: [u8; 32], value: f32) {
        let mut new_node = None;
        let new_idx = self.nodes.len();
        {
            let node = self.nodes.get_mut(idx).unwrap();
            let mut cmp = 0;
            let len = node.name.len();
            for (i, byte) in name.iter().enumerate() {
                if i >= len {
                    break;
                }

                if byte < &node.name[i] {
                    cmp = -1;
                    break;
                } else if byte > &node.name[i] {
                    cmp = 1;
                    break;
                }
            }

            if cmp < 0 {
                if let Some(left_idx) = node.l_child {
                    self.update_node(left_idx, name, value);
                } else {
                    new_node.replace(Node::new(name, value));
                    node.l_child.replace(new_idx);
                }
            } else if cmp > 0 {
                if let Some(right_idx) = node.r_child {
                    self.update_node(right_idx, name, value);
                } else {
                    new_node.replace(Node::new(name, value));
                    node.r_child.replace(new_idx);
                }
            } else {
                node.update_state(value);
            }
        }

        if let Some(node) = new_node {
            self.nodes.push(node);
        }
    }

    fn cities(&self) -> Vec<String> {
        let root = self.nodes[0];
        let mut nodes = Vec::with_capacity(self.nodes.len());
        self.add_children(root, &mut nodes);

        nodes
            .iter()
            .map(|node| {
                String::from_utf8_lossy(
                    &node.name[0..node
                        .name
                        .iter()
                        .position(|b| *b == 0)
                        .unwrap_or(node.name.len())],
                )
                .to_string()
            })
            .collect()
    }

    fn add_children(&self, node: Node, nodes: &mut Vec<Node>) {
        if let Some(l_idx) = node.l_child {
            let lhs = self.nodes[l_idx];
            self.add_children(lhs, nodes);
        }

        nodes.push(node);

        if let Some(r_idx) = node.r_child {
            let rhs = self.nodes[r_idx];
            self.add_children(rhs, nodes);
        }
    }
}
