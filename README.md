## Logical clocks

`clocks` implements some of the modern logical clocks (vector clocks and dotted version vector).

A logical clock is a mechanism for capturing chronological and causal relationships(cause and effect, Event A caused event B, also called as *happened-before* relation) in a distributed system.

Given any two events across multiple nodes in the distributed system, logical clocks help in answering queries like "Does event A *happened-before* B" or "Is event B *concurrent* to event A"

Implementation of dotted version vector is based on the paper (Scalable and Accurate Causality Tracking for Eventually Consistent Stores)[https://haslab.uminho.pt/tome/files/dvvset-dais.pdf]

## Vector clocks vs Version Vectors
Although they both have same data structure representation, they solve different problems.

Vector clocks are used to partial order between any two events in the distributed systems, where as Version Vectors are used to partial order only events that changes datum(say you want to keep multiple versions of same key that are updated concurrently).

For more details about the differences, there good article (here)[https://haslab.wordpress.com/2011/07/08/version-vectors-are-not-vector-clocks/]

## Usage (Vector Clocks)
```rust
fn main() {
	// say we have three actors A, B and C(source of concurrency).
	
    // Case 0: v1 happened_before v2
    // [2, 3, 2]
    let v1 = VectorClock::new()
        .increment("A")
        .increment("A")
        .increment("B")
        .increment("B")
        .increment("B")
        .increment("C")
        .increment("C");

    // [2, 4, 2]
    let v2 = VectorClock::new()
        .increment("A")
        .increment("A")
        .increment("B")
        .increment("B")
        .increment("B")
        .increment("B")
        .increment("C")
        .increment("C");
    assert!(v1.happened_before(&v2));
    assert!(!v2.happened_before(&v1));

    // Case 1: Concurrent
    // [2, 3, 2]
    let v1 = VectorClock::new()
        .increment("A")
        .increment("A")
        .increment("B")
        .increment("B")
        .increment("B")
        .increment("C")
        .increment("C");

    // [1, 4, 1]
    let v2 = VectorClock::new()
        .increment("A")
        .increment("B")
        .increment("B")
        .increment("B")
        .increment("B")
        .increment("C");
    assert!(!v1.happened_before(&v2));
    assert!(!v2.happened_before(&v1));
}
```

## Usage (Dotted Version Vectors)
```rust
    // Case 0: v2 descends v1
    // [2, 3, 2]
    let v1 = VersionVector::new().increment("A").increment("A").increment("B").increment("B").increment("B").increment("C").increment("C");
	
    // [3, 4, 2]
    let v2 = VersionVector::new().increment("A").increment("B").increment("B").increment("C");
    assert!(v1.descends(&v2));
    assert!(!v2.descends(&v1));
	
    // Case 1: Concurrent
    // [2, 3, 2]
    let v1 = VersionVector::new().increment("A").increment("A").increment("B").increment("B").increment("B").increment("C").increment("C");

    // [1, 4, 1]
    let v2 = VersionVector::new().increment("A").increment("B").increment("B").increment("B").increment("B").increment("C");
    assert!(!v1.descends(&v2));
    assert!(!v2.descends(&v1)); // neither v2 descends Case

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

## TODO
[] - working example for dotted version vectors (may be replication of delta-CRDT map?)
[] - make it as rust crate
