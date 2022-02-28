use crate::Node;
pub struct Pool {
    nodes: Vec<Box<Node>>,
    size: usize,
}

impl Pool {
    pub fn new(cap: usize) -> Pool {
        let mut p = Pool {
            nodes: Vec::new(),
            size: cap,
        };
        p.nodes.resize(cap, Box::new(Node::new()));
        p
    }

    pub fn pop(&mut self) -> Box<Node> {
        if self.nodes.is_empty() {
            self.nodes.resize(self.size, Box::new(Node::new()));
        }
        self.nodes.pop().unwrap()
    }

    pub fn push(&mut self, mut ptr: Box<Node>) {
        ptr.children.drain(..).map(|n| self.push(n));
        ptr.reinit();
        self.nodes.push(ptr);
    }
}
