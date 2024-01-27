const HMAX: usize = 256;
const NYT: usize = HMAX; // not yet transmitted

#[derive(Copy, Clone)]
struct Node {
    weight: usize,
    symbol: usize,
}

pub struct Huffman {
    transmitted: [Option<Node>; HMAX + 1],
    parents: [Option<usize>; HMAX + 1],
    lefts: [Option<usize>; HMAX + 1],
    rights: [Option<usize>; HMAX + 1],
}

impl Huffman {
    pub fn new() -> Self {
        Self {
            transmitted: [None; HMAX + 1],
            parents: [None; HMAX + 1],
            lefts: [None; HMAX + 1],
            rights: [None; HMAX + 1],
        }
    }

    pub fn adaptive_encode(&mut self, data: &[u8]) -> Vec<u8> {
        let mut out: [u8; 2 << 16] = [0; 2 << 16];
        let mut bloc = 16;

        out[0] = (data.len() >> 8).try_into().unwrap();
        out[1] = (data.len() & 0xFF).try_into().unwrap();

        for byte in data {
            self.transmit(&(*byte as usize), &mut out, &mut bloc);
            self.add_ref(&(*byte as usize));
        }

        println!("{:?}", &out[..100]);

        vec![]
    }

    pub fn transmit(&mut self, byte: &usize, out: &mut [u8], bloc: &mut usize) {
        match self.transmitted[*byte] {
            Some(node_for_byte) => {
                self.send(&byte, None, out, bloc);
            }
            None => {
                self.transmit(&NYT, out, bloc);
                for i in (0..8).rev() {
                    self.add_bit((byte >> i) & 1, out, bloc);
                }
            }
        }
    }

    pub fn send(
        &mut self,
        byte: &usize,
        child_index: Option<usize>,
        out: &mut [u8],
        bloc: &mut usize,
    ) {
        match self.parents[*byte] {
            Some(parent) => {
                self.send(&parent, Some(*byte), out, bloc);
            }
            None => {
                // do nothing
            }
        }

        if let Some(child_index) = child_index {
            if self.rights[child_index] == Some(child_index) {
                self.add_bit(1, out, bloc);
            } else {
                self.add_bit(0, out, bloc);
            }
        }
    }

    pub fn add_bit(&mut self, bit: usize, out: &mut [u8], bloc: &mut usize) {
        let mut x = 0;
        let mut y = 0;

        y = *bloc >> 3;
        x = *bloc & 7;
        *bloc += 1;

        if x == 0 {
            out[y] = 0;
        }
        out[y] |= (bit as u8) << x;
    }

    pub fn add_ref(&mut self, byte: &usize) {
        match self.transmitted[*byte] {
            Some(byte) => {}
            None => {
                self.transmitted[*byte] = Some(Node {
                    weight: 0,
                    symbol: *byte,
                });
            }
        }
    }
}
