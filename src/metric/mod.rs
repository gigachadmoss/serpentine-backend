pub mod storage;

#[derive(Clone, Debug, PartialEq)]
pub enum MetricValue {
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    // e.g. "engine.rpm"
    pub id: String,
    pub timestamp: i64,
    pub value: MetricValue,
}
