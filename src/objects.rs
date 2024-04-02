use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder as Builder;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Builder)]
pub struct User {
    pub id: u64,
}
