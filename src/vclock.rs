use std::collections::HashMap;
use std::collections::HashSet;

struct VectorClock {
    vector: HashMap<String, i64>,
    // TODO(kavi): Add support mutex for thread-safe?
}

impl VectorClock {
    fn new() -> VectorClock {
        VectorClock {
            vector: HashMap::new(),
        }
    }

    fn increment(mut self, node_id: &str) -> Self {
        self.vector
            .entry(node_id.to_string())
            .and_modify(|e| *e += 1)
            .or_insert(1);
        self
    }

    fn happened_before(&self, w: &VectorClock) -> bool {
        // happened_before check partial order between two vector clocks given.
        // If v *happens-before* w iff for every element i in v should be less than equal
        // to its corresponding element in w And at least one element should be strictly smaller
        // https://en.wikipedia.org/wiki/Vector_clock
        let keys = VectorClock::all_keys(&[&self.vector, &w.vector]);

	let mut sc = 0;

        for k in keys.iter() {
            let v1 = match self.vector.get(k) {
                None => 0,
                Some(v) => *v,
            };
            let v2 = match w.vector.get(k) {
                None => 0,
                Some(v) => *v,
            };

	    if v1 > v2 {
		return false
	    }

            if v1 < v2 {
                sc +=1;
            }
        }
        return sc > 0
    }

    fn concurrent(&self, w: &VectorClock) -> bool {
        !(self.happened_before(w) || w.happened_before(self))
    }

    /// merges the two given vectors via point-wise max.
    fn merge(&self, w: &VectorClock) -> VectorClock {
        let slice = vec![&self.vector, &w.vector];
        let keys = VectorClock::all_keys(&slice[..]);
        let mut res: HashMap<String, i64> = HashMap::new();
        println!("keys: {:?}", keys);
        for k in keys.iter() {
            let e1 = match self.vector.get(k) {
                None => 0,
                Some(v) => *v,
            };
            let e2 = match w.vector.get(k) {
                None => 0,
                Some(v) => *v,
            };

            res.insert(k.to_string(), std::cmp::max(e1, e2));
        }

        VectorClock { vector: res }
    }

    fn all_keys(clocks: &[&HashMap<String, i64>]) -> HashSet<String> {
        let mut res = HashSet::new();

        for clock in clocks {
            for (k, _) in clock.iter() {
                res.insert(k.to_string());
            }
        }
        res
    }
}

#[test]
fn test_vv_new() {
    let mut vv = VectorClock::new();
    vv = vv.increment("A").increment("B");

    assert_eq!(vv.vector.get("A").unwrap(), &1_i64);
    assert_eq!(vv.vector.get("B").unwrap(), &1_i64);

    vv = vv.increment("A").increment("C");

    assert_eq!(vv.vector.get("A").unwrap(), &2_i64);
    assert_eq!(vv.vector.get("C").unwrap(), &1_i64);
}

#[test]
fn test_vv_merge() {
    // [2, 1]
    let v1 = VectorClock::new()
        .increment("A")
        .increment("A")
        .increment("B");
    // [1, 2]
    let v2 = VectorClock::new()
        .increment("B")
        .increment("B")
        .increment("A");

    let v3 = v1.merge(&v2);

    // [2, 2]
    assert_eq!(v3.vector.get("A").unwrap(), &2_i64);
    assert_eq!(v3.vector.get("B").unwrap(), &2_i64);
}

#[test]
fn test_vv_happened_before() {
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

#[test]
fn test_vv_concurrent() {
    // Case 0: not concurrent
    // [2, 3, 2]
    let v1 = VectorClock::new()
        .increment("A")
        .increment("A")
        .increment("B")
        .increment("B")
        .increment("B")
        .increment("C")
        .increment("C");

    // [3, 4, 2]
    let v2 = VectorClock::new()
        .increment("A")
        .increment("B")
        .increment("B")
        .increment("C");
    assert!(!v1.concurrent(&v2));
    assert!(!v2.concurrent(&v1));

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
    assert!(v1.concurrent(&v2));
    assert!(v2.concurrent(&v1));
}
