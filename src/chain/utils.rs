use sha2::{Sha256, Digest};



pub const PROOF_OF_WORK_DIFFICULTY: u8 = 4;

pub struct HashedData{
    hash: [u8;32],
}

impl HashedData {
    pub fn new(data: &[u8]) -> Self{
        let new_data: Result<[u8; 32], _> = data.try_into().map_err(|_| "Slice has a different length than 32");
        match new_data {
            Ok(arr) => Self{ hash: arr },
            Err(e) => panic!("Couldnt convert data to fit into HashedData object\nError: {e}")
        }
    }
    pub fn from_string(str: &String) -> Self{
        let mut hasher = Sha256::new();
        hasher.update(str.as_bytes());
        let hash_result = hasher.finalize(); // GenericArray<u8, 32>
        Self {
            hash: hash_result.into() // GenericArray to [u8; 32]
        }
    }
    
    pub fn get_hash(&self) -> [u8; 32] {
        self.hash
    }

    pub fn hash_as_string(&self) -> String{
        let mut return_str = String::new();
        return_str.push(self.hash[0] as char);

        return_str
    }
}

