use logical_clock::{VersionVector, Dot};
use std::collections::HashMap;

type Key = String;

#[derive(Clone, Debug)]
struct Value{
    val:i64,
    dot:Dot
}

struct KVStore {
    store:HashMap<Key, Vec<Value>>,
    vv:VersionVector,
	
}

impl KVStore {
    fn new() -> KVStore {
	KVStore{
	    store: HashMap::new(),
	    vv: VersionVector::new(),
	}
    }

    fn get(&self, key: &str) -> (Option<Vec<Value>>, VersionVector) {
	match self.store.get(key) {
	    None => (None, self.vv.clone()),
	    Some(v) => (Some(v.clone()), self.vv.clone())
	}
    }

    fn set(mut self, client_id:&str, context: &VersionVector, key: &str, val: i64) -> Self{
	// if incoming request context descends from local clock, just overwrite.
	if context.descends(&self.vv) {
	    self.vv = self.vv.inc(client_id);
	    let dot = self.vv.get_dot(client_id);
	    let new_obj = Value{val: val, dot: dot};
	    
	    // overwrite all the siblings
	    self.store.insert(key.to_string(), vec![new_obj]);
	    return self
	}

	let mut frontier = self.vv.merge(&context);
	frontier = frontier.inc(client_id);
	let dot = frontier.get_dot(client_id);
	let new_obj = Value{val: val, dot: dot};
	self.vv = frontier;
	return self.merge_siblings(key, new_obj)
    }

    fn merge_siblings(mut self, key: &str, new_val: Value) -> Self{
	// replace values that dominated by given value's dot
	let (old, _) = self.get(key);

	match old {
	    None => {
		self.store.insert(key.to_string(), vec![new_val]);
		return self
	    },
	    Some(values) => {
		let mut updated = Vec::new();
		for v in values {
		    if new_val.dot.descends(&v.dot) {
			continue;
		    }
		    updated.push(v);
		}
		updated.push(new_val);
		self.store.insert(key.to_string(), updated);
		return self
	    }
	}
    }
}

fn main() {
    let mut kv = KVStore::new();

    // always get before put - Semantics followed in any High Available Key value store

    // Client A and Client B
    let (_, ctx_a) = kv.get("x");
    let (_, ctx_b) = kv.get("x");

    
    kv = kv.set("A", &ctx_a, "x", 10); // A try to write x=10 with empty context
    kv = kv.set("B", &ctx_b, "x", 15); // B try to write x=12 with same empty context

    // both are concurrent from the client views, so both values should be kept
    assert_eq!(2, kv.store["x"].len());

    // Client C comes in.
    let (_, ctx_c) = kv.get("x");
    // now client C knows all the causal contex, so it replaces the key with all causal past.
    kv = kv.set("C", &ctx_c, "x", 20);
    assert_eq!(1, kv.store["x"].len());
    
    // now B set with old empty context.
    kv = kv.set("B", &ctx_b, "x", 30); // I know contex is empty just set it as 30.

    // From client views latest B write is concurrent to C latest write. so both should be kept.
    assert_eq!(2, kv.store["x"].len());
    
    for (k, v) in kv.store {
	println!("key: {}, values: {:?}", k, v)
	    // val: {}, dot: {:?}", k, v, v.dot);
    }
    println!("vv: {:?}", kv.vv);

}
