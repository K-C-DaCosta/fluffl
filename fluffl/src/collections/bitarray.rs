
pub struct BitArray {
    blocks: Vec<u32>,
    len: u128,
}

#[allow(dead_code)]
impl BitArray {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            len: 0,
        }
    }
    /// attemps to allocate approximately `num_bits` bits. You will ofeten get slightly more bits than requested.
    /// new bits will be set to `bit`. If `num_bits` bits is already allocated, then this function does nothing.
    pub fn allocate(&mut self, num_bits: u128, bit: u32) {
        let extra_block = if num_bits % 32 != 0 { 1 } else { 0 };
        let num_blocks = num_bits as usize / 32 + extra_block;
        let block_val = if bit != 0 { !0 } else { 0 };
        while self.blocks.len() < num_blocks {
            self.blocks.push(block_val);
        }
        self.len = num_bits;
    }

    /// returns number of bits available
    pub fn len(&self) -> u128 {
        self.len
    }

    /// returns number of bits available
    pub fn available_bits(&self) -> u128 {
        self.blocks.len() as u128 * 32
    }

    ///gets bit *at* index and returns either 0 or 1
    pub fn get_bit(&self, index: u128) -> u32 {
        let block_index = (index / 32) as usize;
        let block_bit_index = (index % 32) as u32;
        let bit_block = self.blocks[block_index];
        (bit_block >> block_bit_index) & 1
    }

    ///sets bit of value `bit` *at* `index`  
    pub fn set_bit(&mut self, index: u128, bit: u32) {
        let set_mask = (bit & 1) * (!0);

        let block_index = (index / 32) as usize;
        let block_bit_index = (index % 32) as u32;
        let block = self.blocks[block_index];

        let bit_mask = 1 << block_bit_index;
        let block_set = block | bit_mask;
        let block_unset = block & !bit_mask;

        self.blocks[block_index] = (set_mask & block_set) | ((!set_mask) & block_unset);
    }
}

impl std::fmt::Display for BitArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        if self.len >= 1 {
            write!(f, "{}", self.get_bit(0))?;
        }
        for k in 1..self.len() {
            write!(f, ",{}", self.get_bit(k))?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl std::ops::Not for BitArray {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for bit_block in self.blocks.iter_mut() {
            *bit_block = !*bit_block;
        }
        self
    }
}

#[test]
fn set_bit(){
     let mut bits = BitArray::new();
    bits.allocate(16, 0);
    bits.set_bit(0, 1);
    bits.set_bit(2, 1);
    bits.set_bit(4, 1);
    bits.set_bit(6, 1);
    bits.set_bit(8, 1);
}


