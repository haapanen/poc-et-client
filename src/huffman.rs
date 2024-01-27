use std::{cell::RefCell, rc::Rc};

const HMAX: usize = 256;
const NYT: usize = HMAX;
const INTERNAL_NODE: usize = HMAX + 1;

#[derive(Debug, Clone)]
struct Node {
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
    parent: Option<Rc<RefCell<Node>>>,
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Rc<RefCell<Node>>>,
    head: Option<Rc<RefCell<Node>>>,
    weight: u32,
    symbol: u32,
}

impl Node {
    fn new_ref() -> Rc<RefCell<Node>> {
        Rc::new(RefCell::new(Node {
            left: None,
            right: None,
            parent: None,
            next: None,
            prev: None,
            head: None,
            weight: 0,
            symbol: 0,
        }))
    }
}

pub struct Huffman {
    tree: Option<Rc<RefCell<Node>>>,
    lhead: Option<Rc<RefCell<Node>>>,
    ltail: Option<Rc<RefCell<Node>>>,
    loc: [Option<Rc<RefCell<Node>>>; HMAX + 1],

    bloc: u32,
}

const ARRAY_REPEAT_VALUE: std::option::Option<std::rc::Rc<std::cell::RefCell<Node>>> = None;

impl Huffman {
    pub fn new() -> Huffman {
        Self {
            tree: None,
            lhead: None,
            ltail: None,
            loc: [ARRAY_REPEAT_VALUE; HMAX + 1],
            bloc: 0,
        }
    }

    pub fn adaptive_encode(&mut self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return vec![];
        }

        let head = Node::new_ref();

        self.tree = Some(head.clone());
        self.lhead = Some(head.clone());
        self.loc[NYT] = Some(head.clone());

        head.borrow_mut().symbol = NYT as u32;
        head.borrow_mut().weight = 0;
        head.borrow_mut().next = None;
        head.borrow_mut().prev = None;
        head.borrow_mut().left = None;
        head.borrow_mut().right = None;
        head.borrow_mut().parent = None;
        self.loc[NYT] = self.tree.clone();

        let mut result: [u8; 65536] = [0; 65536];
        result[0] = (data.len() >> 8) as u8;
        result[1] = (data.len() & 0xff) as u8;

        self.bloc = 16;

        for byte in data {
            self.transmit((*byte) as u32, &mut result);
            self.add_ref(*byte as u32);
        }

        return vec![];
    }

    fn transmit(&mut self, symbol: u32, result: &mut [u8; 65536]) {
        match self.loc[symbol as usize].clone() {
            Some(node) => {
                self.send(node.clone(), None, result);
            }
            None => {
                self.transmit(NYT as u32, result);
                for i in (0..8).rev() {
                    self.add_bit((symbol >> i) & 0x1, result);
                }
            }
        }
    }

    fn send(
        &mut self,
        node: Rc<RefCell<Node>>,
        child: Option<Rc<RefCell<Node>>>,
        result: &mut [u8; 65536],
    ) {
        let parent = node.borrow_mut().parent.clone();
        if let Some(parent) = parent {
            self.send(parent.clone(), Some(node.clone()), result);
        }

        if let Some(child) = child {
            let right = node.borrow().right.clone();
            let is_right_child = right
                .as_ref()
                .map_or(false, |right_node| Rc::ptr_eq(right_node, &child));

            if is_right_child {
                self.add_bit(1, result);
            } else {
                self.add_bit(0, result);
            }
        }
    }

    fn add_bit(&mut self, bit: u32, result: &mut [u8; 65536]) {
        let y = self.bloc >> 3;
        self.bloc += 1;
        let x = self.bloc & 0x7;

        if x == 0 {
            result[y as usize] = 0;
        }
        result[y as usize] |= (bit << x) as u8;
    }

    fn add_ref(&mut self, symbol: u32) {
        if let Some(node) = self.loc[symbol as usize].clone() {
            self.increment(Some(node.clone()));
        } else {
            let mut tnode = Node::new_ref();
            let mut tnode2 = Node::new_ref();

            tnode2.borrow_mut().symbol = INTERNAL_NODE as u32;
            tnode2.borrow_mut().weight = 1;
            tnode2.borrow_mut().next = self.lhead.as_mut().unwrap().borrow_mut().next.clone();
            if self.lhead.as_mut().unwrap().borrow_mut().next.is_some() {
                self.lhead
                    .as_mut()
                    .unwrap()
                    .borrow_mut()
                    .next
                    .as_mut()
                    .unwrap()
                    .borrow_mut()
                    .prev = Some(tnode2.clone());

                if self.lhead.as_ref().unwrap().borrow_mut().weight == 1 {
                    tnode2.borrow_mut().head = self
                        .lhead
                        .as_mut()
                        .unwrap()
                        .borrow_mut()
                        .next
                        .as_mut()
                        .unwrap()
                        .borrow_mut()
                        .head
                        .clone();
                } else {
                    tnode2.borrow_mut().head = Some(tnode2.clone());
                }
            } else {
                tnode2.borrow_mut().head = Some(tnode2.clone());
            }

            if let Some(lhead) = self.lhead.clone() {
                lhead.borrow_mut().head = Some(tnode2.clone());
                tnode2.borrow_mut().prev = Some(lhead.clone());

                tnode.borrow_mut().symbol = symbol;
                tnode.borrow_mut().weight = 1;
                tnode.borrow_mut().next = lhead.borrow_mut().next.clone();

                if let Some(next) = lhead.borrow_mut().next.clone() {
                    next.borrow_mut().prev = Some(tnode.clone());
                    if next.as_ref().borrow().weight == 1 {
                        tnode.borrow_mut().head = next.as_ref().borrow_mut().head.clone();
                    } else {
                        tnode.borrow_mut().head = Some(tnode2.clone());
                    }
                } else {
                    tnode.borrow_mut().head = Some(tnode.clone());
                }

                lhead.borrow_mut().next = Some(tnode.clone());
                tnode.borrow_mut().prev = Some(lhead.clone());
                tnode.borrow_mut().left = None;
                tnode.borrow_mut().right = None;

                if let Some(parent) = lhead.borrow_mut().parent.clone() {
                    let is_left_child;
                    {
                        // Limiting the scope of the first mutable borrow
                        let parent_borrow = parent.borrow();
                        if let Some(left) = parent_borrow.left.clone() {
                            is_left_child = Rc::ptr_eq(&left, &lhead);
                        } else {
                            is_left_child = false; // or handle this case as needed
                        }
                    } // The mutable borrow ends here

                    // Now it's safe to borrow parent mutably again
                    let mut parent_borrow_mut = parent.borrow_mut();
                    if is_left_child {
                        parent_borrow_mut.left = Some(tnode2.clone());
                    } else {
                        parent_borrow_mut.right = Some(tnode2.clone());
                    }
                } else {
                    self.tree = Some(tnode2.clone());
                }

                tnode2.borrow_mut().right = Some(tnode.clone());
                tnode2.borrow_mut().left = Some(lhead.clone());

                tnode2.borrow_mut().parent = lhead.borrow_mut().parent.clone();
                lhead.borrow_mut().parent = Some(tnode2.clone());
                tnode.borrow_mut().parent = Some(tnode2.clone());

                self.loc[symbol as usize] = Some(tnode.clone());

                self.increment(tnode2.borrow_mut().parent.clone());
            }
        }
    }

    fn increment(&mut self, node: Option<Rc<RefCell<Node>>>) {
        let mut lnode: Option<Rc<RefCell<Node>>> = None;

        if let Some(node) = node.clone() {
            if let Some(next) = node.borrow_mut().next.clone() {
                if node.borrow().weight == next.borrow().weight {
                    lnode = node.borrow().head.clone();
                    if Huffman::is_same_node(lnode.clone(), node.borrow().parent.clone()) {
                        self.swap(lnode.clone(), Some(node.clone()));
                    }

                    self.swap_list(lnode.clone(), Some(node.clone()));
                }
            }

            // if let Some(prev) = node.borrow_mut().prev.clone() {
            //     if prev.borrow().weight == node.borrow().weight {
            //         node.borrow_mut().head = Some(prev.clone());
            //     } else {
            //         node.borrow_mut().head = None;
            //     }
            // }

            let prev_clone = node.borrow().prev.clone();

            if let Some(prev) = prev_clone {
                // Determine the condition outside of the mutable borrow of node
                let should_link_to_prev = prev.borrow().weight == node.borrow().weight;

                // Now perform the mutable borrow once
                let mut node_borrow_mut = node.borrow_mut();
                if should_link_to_prev {
                    node_borrow_mut.head = Some(prev);
                } else {
                    node_borrow_mut.head = None;
                }
            }

            node.borrow_mut().weight += 1;

            if let Some(next) = node.borrow_mut().next.clone() {
                if node.borrow().weight == next.borrow().weight {
                    node.borrow_mut().head = next.borrow_mut().head.clone();
                } else {
                    node.borrow_mut().head = None;
                }
            }

            if let Some(parent) = node.borrow_mut().parent.clone() {
                self.increment(Some(parent.clone()));

                if let Some(prev) = node.borrow_mut().prev.clone() {
                    if Huffman::is_same_node(Some(prev.clone()), Some(parent.clone())) {
                        self.swap_list(Some(node.clone()), Some(parent.clone()));
                        if Huffman::is_same_node(node.borrow().head.clone(), Some(node.clone())) {
                            node.borrow_mut().head = Some(parent.clone());
                        }
                    }
                }
            }
        }
    }

    fn swap(&mut self, node1: Option<Rc<RefCell<Node>>>, node2: Option<Rc<RefCell<Node>>>) {
        let node1 = node1.unwrap();
        let node2 = node2.unwrap();

        let parent1 = node1.borrow_mut().parent.clone();
        let parent2 = node2.borrow_mut().parent.clone();

        if let Some(parent1) = parent1.clone() {
            if Huffman::is_same_node(parent1.borrow().left.clone(), Some(node1.clone())) {
                parent1.borrow_mut().left = Some(node2.clone());
            } else {
                parent1.borrow_mut().right = Some(node2.clone());
            }
        } else {
            self.tree = Some(node2.clone());
        }

        if let Some(parent2) = parent2.clone() {
            if Huffman::is_same_node(parent2.borrow().left.clone(), Some(node2.clone())) {
                parent2.borrow_mut().left = Some(node1.clone());
            } else {
                parent2.borrow_mut().right = Some(node1.clone());
            }
        }

        node1.borrow_mut().parent = parent2.clone();
        node2.borrow_mut().parent = parent1.clone();
    }

    fn swap_list(&mut self, node1: Option<Rc<RefCell<Node>>>, node2: Option<Rc<RefCell<Node>>>) {
        let node1 = node1.unwrap();
        let node2 = node2.unwrap();

        let mut parent1 = node1.borrow_mut().next.clone();
        node1.borrow_mut().next = node2.borrow_mut().next.clone();
        node2.borrow_mut().next = parent1.clone();

        parent1 = node1.borrow_mut().prev.clone();
        node1.borrow_mut().prev = node2.borrow_mut().prev.clone();
        node2.borrow_mut().prev = parent1.clone();

        if Huffman::is_same_node(node1.borrow().next.clone(), Some(node1.clone())) {
            node1.borrow_mut().next = Some(node2.clone());
        }
        if Huffman::is_same_node(node2.borrow().next.clone(), Some(node2.clone())) {
            node2.borrow_mut().next = Some(node1.clone());
        }
        if let Some(next) = node1.clone().borrow_mut().next.clone() {
            next.borrow_mut().prev = Some(node1.clone());
        }
        if let Some(next) = node2.clone().borrow_mut().next.clone() {
            next.borrow_mut().prev = Some(node2.clone());
        }
        if let Some(prev) = node1.clone().borrow_mut().prev.clone() {
            prev.borrow_mut().next = Some(node1.clone());
        }
        if let Some(prev) = node2.clone().borrow_mut().prev.clone() {
            prev.borrow_mut().next = Some(node2.clone());
        }
    }

    fn is_same_node(node1: Option<Rc<RefCell<Node>>>, node2: Option<Rc<RefCell<Node>>>) -> bool {
        if node1.is_none() || node2.is_none() {
            return false;
        }

        let node1 = node1.unwrap();
        let node2 = node2.unwrap();

        Rc::ptr_eq(&node1, &node2)
    }
}
