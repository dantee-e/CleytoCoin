pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub ammount: f32,
}

impl Transaction {
    fn new(ammount:f32) -> Self{
        Self{
            sender: String::new(),
            receiver: String::new(),
            ammount: ammount
        }
    }
}