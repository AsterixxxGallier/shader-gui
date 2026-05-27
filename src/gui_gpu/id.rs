use std::collections::HashMap;
use rand::random;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Id(u64);

impl Id {
    pub fn new() -> Self {
        Self(random())
    }
}

pub type IdMap<V> = HashMap<Id, V>;