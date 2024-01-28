use std::path::Display;

const MAX_SYMBOLS: usize = 256;
const NYT: usize = MAX_SYMBOLS; // not yet transmitted
const INTERNAL_NODE: usize = MAX_SYMBOLS + 1;
const MAX_NODES: usize = 2 * MAX_SYMBOLS;

#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub weight: usize,
    pub symbol: usize,

    pub parent: *mut Node,
    pub left: *mut Node,
    pub right: *mut Node,
    pub next: *mut Node,
    pub prev: *mut Node,
    pub head: *mut *mut Node,
}

impl Node {
    fn new() -> Self {
        Node {
            weight: 0,
            symbol: 0,
            parent: std::ptr::null_mut(),
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
            head: std::ptr::null_mut(),
        }
    }
}

fn print_node(node: *const Node) {
    if node.is_null() {
        println!("node is null");
        return;
    }

    unsafe {
        println!(
            "Node {} at {:p} has weight {}",
            (*node).symbol,
            node,
            (*node).weight
        );
        println!(
            "left: {:p} right: {:p} parent: {:p} next: {:p} prev: {:p} head: {:p}",
            (*node).left,
            (*node).right,
            (*node).parent,
            (*node).next,
            (*node).prev,
            (*node).head
        );
    }
}

fn print_node_ptr(node: *mut *mut Node) {
    unsafe {
        println!("node ptr: {:p}", node);
    }
}

fn print_nodes(nodes: [Node; MAX_NODES], len: usize) {
    for node in nodes[..len].iter() {
        unsafe {
            print_node(node as *const Node);
        }
    }
}

pub struct Huffman {
    nodes: [Node; MAX_NODES],
    next_free_node_index: usize,
    node_ptrs: [*mut Node; MAX_NODES],
    next_free_node_ptr_index: usize,

    freed_pp_nodes: Vec<*mut *mut Node>,

    tree: *mut Node,
    lhead: *mut Node,
    ltail: *mut Node,
    symbols: [*mut Node; MAX_SYMBOLS + 1],

    bloc: usize,
}

impl Huffman {
    pub fn new() -> Self {
        Huffman {
            nodes: [Node::new(); MAX_NODES],
            next_free_node_index: 0,
            node_ptrs: [std::ptr::null_mut(); MAX_NODES],
            next_free_node_ptr_index: 0,
            freed_pp_nodes: Vec::new(),
            tree: std::ptr::null_mut(),
            lhead: std::ptr::null_mut(),
            ltail: std::ptr::null_mut(),
            symbols: [std::ptr::null_mut(); MAX_SYMBOLS + 1],
            bloc: 0,
        }
    }

    pub fn adaptive_compress(&mut self, input: &[u8]) -> Vec<u8> {
        let mut result: [u8; 65536] = [0; 65536];

        unsafe {
            let mut node = self.get_free_node();
            self.tree = node;
            self.lhead = node;
            self.symbols[NYT] = node;

            (*node).symbol = NYT;
            (*node).weight = 0;
            (*self.lhead).next = std::ptr::null_mut();
            (*self.lhead).prev = std::ptr::null_mut();
            (*self.tree).parent = std::ptr::null_mut();
            (*self.tree).left = std::ptr::null_mut();
            (*self.tree).right = std::ptr::null_mut();
            self.symbols[NYT] = self.tree;
        }

        result[0] = (input.len() >> 8) as u8;
        result[1] = (input.len() & 0xFF) as u8;

        self.bloc = 16;

        for &byte in input {
            self.transmit(byte as u32, &mut result);
            self.add_ref(byte as u32);
        }

        println!("result: {:?}", result[..(self.bloc >> 3)].to_vec());

        result[..(self.bloc >> 3)].to_vec()
    }

    fn get_free_node(&mut self) -> *mut Node {
        let node = &mut self.nodes[self.next_free_node_index] as *mut Node;
        self.next_free_node_index += 1;
        node
    }

    fn get_free_pp_node(&mut self) -> *mut *mut Node {
        if self.freed_pp_nodes.is_empty() {
            let node = &mut self.node_ptrs[self.next_free_node_ptr_index];
            self.next_free_node_ptr_index += 1;
            node
        } else {
            self.freed_pp_nodes.pop().unwrap()
        }
    }

    fn free_node(&mut self, node: *mut *mut Node) {
        self.freed_pp_nodes.push(node);
    }

    fn transmit(&mut self, byte: u32, result: &mut [u8; 65536]) {
        if self.symbols[byte as usize].is_null() {
            self.transmit(NYT as u32, result);

            for i in (0..8).rev() {
                self.add_bit((byte >> i) & 0x1, result);
            }
        } else {
            self.send(self.symbols[byte as usize], std::ptr::null_mut(), result);
        }
    }

    fn add_bit(&mut self, bit: u32, result: &mut [u8; 65536]) {
        let y = self.bloc >> 3;
        let x = self.bloc & 0x7;
        self.bloc += 1;

        if x == 0 {
            result[y] = 0;
        }

        result[y] |= (bit << x) as u8;
    }

    fn send(&mut self, node: *mut Node, child: *mut Node, result: &mut [u8; 65536]) {
        unsafe {
            if !(*node).parent.is_null() {
                self.send((*node).parent, node, result);
            }

            if !child.is_null() {
                if (*node).left == child {
                    self.add_bit(0, result);
                } else {
                    self.add_bit(1, result);
                }
            }
        }
    }

    fn add_ref(&mut self, byte: u32) {
        if self.symbols[byte as usize].is_null() {
            unsafe {
                let tnode = self.get_free_node();
                let tnode2 = self.get_free_node();

                (*tnode2).symbol = INTERNAL_NODE;
                (*tnode2).weight = 1;
                (*tnode2).next = (*self.lhead).next;

                if !(*self.lhead).next.is_null() {
                    (*(*self.lhead).next).prev = tnode2;

                    if (*(*self.lhead).next).weight == 1 {
                        (*tnode2).head = (*(*self.lhead).next).head;
                    } else {
                        (*tnode2).head = self.get_free_pp_node();
                        let head = (*tnode2).head;
                        (*(*(*head)).head) = tnode2;
                    }
                } else {
                    (*tnode2).head = self.get_free_pp_node();
                    *(*tnode2).head = tnode2;
                }

                (*self.lhead).next = tnode2;
                (*tnode2).prev = self.lhead;

                (*tnode).symbol = byte as usize;
                (*tnode).weight = 1;
                (*tnode).next = (*self.lhead).next;

                if !(*self.lhead).next.is_null() {
                    (*(*self.lhead).next).prev = tnode;

                    if (*(*self.lhead).next).weight == 1 {
                        (*tnode).head = (*(*self.lhead).next).head;
                    } else {
                        (*tnode).head = self.get_free_pp_node();
                        *(*tnode).head = tnode2;
                    }
                } else {
                    (*tnode).head = self.get_free_pp_node();
                    *(*tnode).head = tnode2;
                }

                (*self.lhead).next = tnode;
                (*tnode).prev = self.lhead;
                (*tnode).left = std::ptr::null_mut();
                (*tnode).right = std::ptr::null_mut();

                if !(*self.lhead).parent.is_null() {
                    if (*(*self.lhead).parent).left == self.lhead {
                        (*(*self.lhead).parent).left = tnode2;
                    } else {
                        (*(*self.lhead).parent).right = tnode2;
                    }
                } else {
                    self.tree = tnode2;
                }

                (*tnode2).right = tnode;
                (*tnode2).left = self.lhead;

                (*tnode2).parent = (*self.lhead).parent;
                (*self.lhead).parent = tnode2;
                (*tnode).parent = tnode2;

                self.symbols[byte as usize] = tnode;

                self.increment((*tnode2).parent);
            }
        } else {
            self.increment(self.symbols[byte as usize]);
        }
    }

    fn increment(&mut self, node: *mut Node) {
        if node.is_null() {
            return;
        }

        unsafe {
            let mut lnode: *mut Node = std::ptr::null_mut();

            if !(*node).next.is_null() && (*(*node).next).weight == (*node).weight {
                lnode = *(*node).head;

                if lnode != (*node).parent {
                    self.swap(lnode, node);
                }

                self.swap_list(lnode, node);
            }

            if !(*node).prev.is_null() && (*(*node).prev).weight == (*node).weight {
                (*(*node).head) = (*node).prev;
            } else {
                let freed_node = (*node).head;
                (*node).head = std::ptr::null_mut();
                self.free_node(freed_node);
            }

            (*node).weight += 1;

            if !(*node).next.is_null() && (*(*node).next).weight == (*node).weight {
                (*node).head = (*(*node).next).head;
            } else {
                (*node).head = self.get_free_pp_node();
                *(*node).head = node;
            }

            if !(*node).parent.is_null() {
                self.increment((*node).parent);
                if (*node).prev == (*node).parent {
                    self.swap_list(node, (*node).parent);
                    if *(*node).head == node {
                        *(*node).head = (*node).parent;
                    }
                }
            }
        }
    }

    fn swap(&mut self, node1: *mut Node, node2: *mut Node) {
        unsafe {
            let par1 = (*node1).parent;
            let par2 = (*node2).parent;

            if !par1.is_null() {
                if (*par1).left == node1 {
                    (*par1).left = node2;
                } else {
                    (*par1).right = node2;
                }
            } else {
                self.tree = node2;
            }

            if !par2.is_null() {
                if (*par2).left == node2 {
                    (*par2).left = node1;
                } else {
                    (*par2).right = node1;
                }
            } else {
                self.tree = node1;
            }

            (*node1).parent = par2;
            (*node2).parent = par1;
        }
    }

    fn swap_list(&mut self, node1: *mut Node, node2: *mut Node) {
        unsafe {
            let mut par1 = (*node1).next;
            (*node1).next = (*node2).next;
            (*node2).next = par1;

            par1 = (*node1).prev;
            (*node1).prev = (*node2).prev;
            (*node2).prev = par1;

            if ((*node1).next == node1) {
                (*node1).next = node2;
            }
            if ((*node2).next == node2) {
                (*node2).next = node1;
            }
            if (!(*node1).next.is_null()) {
                (*(*node1).next).prev = node1;
            }
            if (!(*node2).next.is_null()) {
                (*(*node2).next).prev = node2;
            }
            if (!(*node1).prev.is_null()) {
                (*(*node1).prev).next = node1;
            }
            if (!(*node2).prev.is_null()) {
                (*(*node2).prev).next = node2;
            }
        }
    }
}
