mod connect4;

const W: usize = 9;
const H: usize = 7;

const POLICY_SIZE: usize = W;
const INPUT_SIZE: usize = H * W * 2;

struct NN_Output {
    p: [f32; POLICY_SIZE],
    v: f32
}

#[derive(Clone)]
pub struct Node {
    leaf: bool,
    visits: i32,
    value: i32,
    Q: f64,
    P: f64,
    children: Vec<Box<Node>>,
}

impl Node {
    fn new()-> Self  {
        Node {
            leaf: false,
            visits: 0,
            value: 0,
            Q: 0.,
            P: 0.,
            children: Vec::new(),
        }
    }

    fn Select(self) -> &'static Box<Node> {
         &self.children[0]
    }
}


pub struct Pool {
    nodes: Vec<Node>,
    ptrs: Vec<usize>
}

impl Pool {
    fn new(cap: usize) -> Pool {
        let node = Node::new();
        let mut p = Pool{
            ptrs: Vec::new(),
            nodes: Vec::new()
        };
        p.nodes.resize(cap, Node::new());
        for i in 0 .. cap {
            p.ptrs.push(i);
        }
        p
    }
    fn pop(mut self) -> Box<Node> {
        self.pointers.pop().unwrap()
    }
    fn push(mut self, b: Box<Node>) {
        self.pointers.push(b);
    }
}

fn main() {
    let mut objects = Vec::new();
    let mut pointers = Vec::new();
    objects.resize(1000000, Node::new());
    for o in objects.iter_mut() {
        pointers.push(o);
    }
    println!("Hello, world!");
}
