use std::collections::HashMap;
use std::fmt::Debug;

pub fn print_hashmap<K: Debug, V: Debug>(hashmap: &HashMap<K, V>) {
    hashmap.iter().for_each(|(k, v)| {
        tracing::info!("k: {:?}, v: {:?}", k, v);
    });
}
