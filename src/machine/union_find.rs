
#[derive(Clone)]
struct Node {
    depth : usize,
    parent : Option<usize>
}

impl Node {
    fn new() -> Node { Node { depth : 0, parent : None } }
}

#[derive(Clone)]
pub struct UnionFind {
    array : Vec<Node>
}

impl UnionFind {
    
    pub fn new() -> UnionFind { UnionFind { array : vec![] } }

    pub fn find(&self, mut i : usize) -> usize {
        assert!(i < self.array.len());
        while let Some(p) = self.array[i].parent { i = p; }
        i
    }
    
    pub fn register(&mut self, i : usize) {
        assert!(i >= self.array.len());
        self.array.resize_with(i+1, || { Node::new() });
    }
    
    pub fn union(&mut self, i : usize, j : usize) {
        let a = self.find(i);
        let b = self.find(j);
        
        if a != b {
            if self.array[a].depth > self.array[b].depth {
                self.array[b].parent = Some(a)
            }
            else if self.array[a].depth < self.array[b].depth {
                self.array[a].parent = Some(b)
            }
            else {
                self.array[a].parent = Some(b);
                self.array[b].depth += 1;
            }
        }
    }
}