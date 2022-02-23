
use std::mem::{ManuallyDrop, forget};
struct Pool {
    objects: [Node; 1000]
}

impl Pool {
    pub fn new(cap: usize) -> Pool {
        let mut objects = Vec::new();      
        objects.reserve(cap);
        objects.resize(cap, Node::new());
        Pool {
            objects: objects
        }
    }

    pub fn pull(&self) -> Reusable {
        self.objects.lock().pop().map(|data | Reusable::new(self, data))

    }
}

pub struct Reusable<'a> {
    pool: &'a Pool,
    data: ManuallyDrop<Node>
}

#[derive(Clone)]
pub struct Node {

}


impl Node {
    fn new()-> Self  {
        Node {}
    }
}

fn main() {
    let pool = Pool::new(100);
    println!("Hello, world!");
}
