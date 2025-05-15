
// TODO: string slice here
#[derive(Debug)]
pub struct DiffItem {
    pub object: String,
    pub diff: String,
}

impl DiffItem {
    pub fn new(object: &str, diff: &str) -> Self {
        Self {
            object: object.to_string(),
            diff: diff.to_string(),
        }
    }
}

