# Logical clocks

[![Package version](https://img.shields.io/crates/v/logical_clock.svg)](https://crates.io/crates/logical_clock)
[![Package docs](https://docs.rs/logical_clock/badge.svg)](https://docs.rs/logical_clock)
[![License](https://img.shields.io/badge/license-MIT%20License-blue.svg)](https://github.com/kavirajk/clock/blob/master/LICENSE)


`clocks` implements some of the modern logical clocks (vector clocks and dotted version vector).

A logical clock is a mechanism for capturing chronological and causal relationships(cause and effect, Event A caused event B, also called as *happened-before* relation) in a distributed system.

Given any two events across multiple nodes in the distributed system, logical clocks help in answering queries like "Does event A *happened-before* B" or "Is event B *concurrent* to event A"

Implementation of dotted version vector is based on the paper [Scalable and Accurate Causality Tracking for Eventually Consistent Stores](https://haslab.uminho.pt/tome/files/dvvset-dais.pdf)

## Vector clocks vs Version Vectors
Although they both have same data structure representation, they solve different problems.

Vector clocks are used to partial order between any two events in the distributed systems, where as Version Vectors are used to partial order only events that changes datum(say you want to keep multiple versions of same key that are updated concurrently).

For more details about the differences, there good article [here](https://haslab.wordpress.com/2011/07/08/version-vectors-are-not-vector-clocks/)

## Usage (A simple Key Value Store, simulating multiple clients)
```rust
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

```

## Logical clocks in Real time
1. Go race detector uses vector clocks to detect data race between go routines. Basic idea is every go routine have its own vector clock and when a shared memory is accessed by multiple goroutines, their vector clocks are compared to find if they are concurrent!
https://www.slideshare.net/InfoQ/looking-inside-a-race-detector

2. Riak Key Value store uses Dotted Version Vector to track concurrent versions of same key in multiple replica.
https://riak.com/posts/technical/vector-clocks-revisited-part-2-dotted-version-vectors/index.html?p=9929.html

## References
1. https://haslab.uminho.pt/tome/files/dvvset-dais.pdf
2. https://github.com/ricardobcl/Dotted-Version-Vectors
3. https://lamport.azurewebsites.net/pubs/time-clocks.pdf

## Licence
MIT
