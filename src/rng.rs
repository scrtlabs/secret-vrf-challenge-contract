use rand_chacha::ChaChaRng;
use rand_core::{CryptoRng, Error, RngCore, SeedableRng};
use sha2::{Digest, Sha256};


pub struct Prng {
    rng: ChaChaRng,
}

impl RngCore for Prng {

    // fn rand_range(&mut self, min: u32, max: u32) -> u32 {
    //     while {
    //         self.rng.next_u32()
    //     }
    // }

    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

impl CryptoRng for Prng {}

impl Prng {
    pub fn new(seed: &[u8]) -> Self {
        let mut hasher = Sha256::new();

        // write input message
        hasher.update(&seed);
        let hash = hasher.finalize();

        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(hash.as_slice());

        let rng = ChaChaRng::from_seed(hash_bytes);

        Self { rng }
    }

    pub fn rand_bytes(&mut self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        self.rng.fill_bytes(&mut bytes);

        bytes
    }

    pub fn set_word_pos(&mut self, count: u32) {
        self.rng.set_word_pos(count.into());
    }
}