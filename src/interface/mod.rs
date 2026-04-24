pub trait Interface {
    type Error: std::error::Error;
    fn init() -> Result<Box<Self>, Self::Error>;
}
