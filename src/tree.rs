// Hash Tree implementation for efficient similarity search using Hamming distance
//
// This module implements a 16-way branching tree structure that stores 64-bit hash values
// and enables fast approximate matching based on Hamming distance (number of differing bits).
//
// Key concepts:
// - Each 64-bit hash is split into 16 chunks of 4 bits each
// - Each tree level uses one 4-bit chunk to decide which of 16 branches to follow
// - The tree has a maximum depth of 16 levels (64 bits / 4 bits per level)
// - During search, branches with similar bit patterns (within max_diff tolerance) are explored
// - This allows finding "similar" hashes without comparing against every stored hash
//
// Time complexity:
// - Insert: O(TREE_DEPTH) = O(16) = O(1)
// - Search: O(BRANCH_FACTOR^max_diff * TREE_DEPTH) in worst case, typically much better

// Configuration constants for the hash tree structure
// The tree breaks down a 64-bit hash into 4-bit chunks, creating a 16-way branching tree
const BRANCH_BITS: usize = 4;                          // Number of bits used per tree level
const BRANCH_FACTOR: u8 = 1 << BRANCH_BITS;            // Number of branches per node (2^4 = 16)
const TREE_DEPTH: usize = 64 / BRANCH_BITS;            // Total tree depth (64 bits / 4 bits per level = 16 levels)

// Represents a branch in the hash tree - either empty or containing a child node
enum HashTreeEntry<T> {
    None,                        // Empty branch (no data in this path)
    Node(Box<HashTree<T>>),      // Contains a subtree (boxed to avoid recursive type sizing issues)
}

impl<T> Default for HashTreeEntry<T> {
    fn default() -> Self {
        HashTreeEntry::None
    }
}

// Extracts the lowest 'bits' from a u64 value and returns (remaining_bits, extracted_bits)
// This is used to progressively consume the hash value as we traverse down the tree
#[inline(always)]
fn pop_bits(value: u64, bits: usize) -> (u64, u64) {
    let mask = (1 << bits) - 1;              // Create a mask for the lowest 'bits' bits
    (value >> bits, value & mask)            // Return (shifted value, extracted bits)
}

// A 16-way branching tree for storing and searching hash values with Hamming distance tolerance
// Each node has 16 branches (one for each possible 4-bit value) and optionally stores a value at leaf nodes
pub struct HashTree<T> {
    branches: [HashTreeEntry<T>; BRANCH_FACTOR as usize],  // 16 branches, one for each 4-bit pattern
    value: Option<T>,                                       // Value stored at leaf nodes only
}

impl<T> HashTree<T> {
    // Creates an empty hash tree node with no branches or values
    pub fn new() -> Self {
        HashTree {
            branches: [const { HashTreeEntry::None }; BRANCH_FACTOR as usize],
            value: None,
        }
    }

    // Creates a leaf node containing a value (used at the bottom of the tree)
    fn leaf(value: T) -> Self {
        HashTree {
            branches: [const { HashTreeEntry::None }; BRANCH_FACTOR as usize],
            value: Some(value),
        }
    }

    // Searches for a hash value in the tree, allowing up to max_diff bit differences (Hamming distance)
    // Returns a reference to the stored value if a match is found within the tolerance
    pub fn contains(&self, hash: u64, max_diff: u8) -> Option<&T> {
        self._contains(hash, max_diff, 0)
    }

    // Recursive implementation of contains that tracks tree depth and remaining allowed differences
    fn _contains(&self, hash: u64, max_diff: u8, level: usize) -> Option<&T> {
        let remaining_levels = TREE_DEPTH - level as usize;

        // Base case: reached a leaf level, return any stored value
        if remaining_levels == 0 {
            return self.value.as_ref();
        }

        // Extract the 4-bit chunk for this level
        let (rest, level_bits) = pop_bits(hash, BRANCH_BITS);

        // Check all 16 branches, but only follow those within our Hamming distance budget
        for i in 0..BRANCH_FACTOR {
            // Calculate how many bits differ between the query and this branch
            let diff = (level_bits as u8 ^ i).count_ones() as u8;

            // Only explore branches where the bit difference doesn't exceed our remaining budget
            if diff <= max_diff {
                match &self.branches[i as usize] {
                    HashTreeEntry::None => {},
                    HashTreeEntry::Node(node) => {
                        // Recursively search this branch with reduced difference budget
                        if let Some(val) = node._contains(rest, max_diff - diff, level + 1) {
                            return Some(val);
                        }
                    }
                }
            }
        }
        None
    }

    // Recursive implementation to insert a value at the position determined by the hash
    fn _add(&mut self, hash: u64, value: T, level: usize) {
        let (rest, level_bits) = pop_bits(hash, BRANCH_BITS);

        // Base case: at the deepest level, create a leaf node with the value
        if level == (TREE_DEPTH - 1) as usize {
            self.branches[level_bits as usize] = HashTreeEntry::Node(HashTree::leaf(value).into());
            return;
        }

        // Recursive case: navigate to the appropriate branch based on the current 4-bit chunk
        match self.branches[level_bits as usize] {
            HashTreeEntry::None => {
                // This branch doesn't exist yet, create a new node and continue insertion
                let mut node = HashTree::new();
                node._add(rest, value, level + 1);
                self.branches[level_bits as usize] = HashTreeEntry::Node(node.into());
            }
            HashTreeEntry::Node(ref mut node) => {
                // Branch exists, recursively insert into the child node
                node._add(rest, value, level + 1);
            }
        }
    }

    // Adds a value to the tree at the position determined by the hash
    pub fn add(&mut self, hash: u64, value: T) {
        self._add(hash, value, 0);
    }

    // Returns the total number of values stored in the tree
    pub fn len(&self) -> usize {
        let mut count = if self.value.is_some() { 1 } else { 0 };

        for branch in &self.branches {
            if let HashTreeEntry::Node(node) = branch {
                count += node.len();
            }
        }

        count
    }
}
