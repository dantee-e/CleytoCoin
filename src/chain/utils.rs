use sha2::{Sha256, Digest};


pub struct HashedData{
    hash: [u8;32],
}

impl HashedData {
    pub fn new(str: &String) -> Self{
        let mut hasher = Sha256::new();
        hasher.update(str.as_bytes());
        let hash_result = hasher.finalize(); // GenericArray<u8, 32>
        Self {
            hash: hash_result.into() // GenericArray to [u8; 32]
        }
    }

    pub fn hash_as_string(&self) -> String{
        let mut return_str = String::new();
        return_str.push(self.hash[0] as char);

        return_str
    }
}

pub struct Date{
    year: u8,
    month: i8,
    day: i8,
    hour: i8,
    minute:i8,
    second: i8,
}

impl Date {
    fn to_string(&self) -> String {
        format!("{}.{}.{}.{}.{}.{}", self.year, self.month, self.day, self.hour, self.minute, self.second)
    }
}