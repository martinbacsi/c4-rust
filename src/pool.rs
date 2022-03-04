use crate::Node;
pub struct Pool {
    nodes: Vec<Node>,
    ptrs: Vec<Box<Node>>,
    size: usize,
}

impl Pool {
    pub fn new(cap: usize) -> Pool {
        let mut p = Pool {
            nodes: Vec::new(),
            ptrs: Vec::new(),
            size: cap,
        };
        p.grow();
        p
    }

    pub fn grow(&mut self) {
        self.nodes.resize(self.size, Node::new());
        for n in self.nodes.iter_mut() {
            let ptr = n as *mut Node;
            unsafe {
                let b = Box::from_raw(ptr);
                self.ptrs.push(b);
            }
        }
    }

    pub fn pop(&mut self) -> Box<Node> {
        self.ptrs.pop().unwrap()
    }

    pub fn push(&mut self, mut ptr: Box<Node>) {
        ptr.children.drain(..).for_each(|n| self.push(n));
        ptr.reinit();
        self.ptrs.push(ptr);
    }
}
