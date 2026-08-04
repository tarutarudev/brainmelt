#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Add(i64),
    Move(i64),
    Put,
    Get,
    Loop(Vec<Node>),
    Clear,
    Transfer {
        offset: i64,
        multiplier: i64,
    },
}
