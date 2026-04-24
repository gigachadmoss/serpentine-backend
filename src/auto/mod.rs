pub trait AutoProvider {
    fn init() -> Self;
    fn pause(&mut self);
    fn resume(&mut self);
}
