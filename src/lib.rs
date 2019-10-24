mod dvv;
mod vclock;
pub use dvv::VersionVector;
pub use vclock::VectorClock;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
