use crate::{parser::SqlParser, types::DiffItem};
use similar::{ChangeTag, TextDiff};


pub struct Differ {
    source: SqlParser,
    target: SqlParser,
}

impl Differ {
    pub fn new(source: SqlParser, target: SqlParser) -> Self {
        Self {
            source,
            target,
        }
    }

    pub fn get_diff_items(&self) -> Vec<DiffItem> {
        let mut diffs = self.get_internal_object_diff();
        diffs.append(&mut self.get_internal_column_object_diff());
        diffs.sort();
        diffs.dedup();

        let mut diff_items = vec![];

        for d in diffs {
            let source_ddl = self.source.get_ddl_by_object(&d).expect("Missing source ddl");
            let target_ddl = self.target.get_ddl_by_object(&d).unwrap_or(String::new());

            let diff = TextDiff::from_lines(&target_ddl, &source_ddl);
            let mut diff_lines = vec![];

            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                diff_lines.push(format!("{}{}", sign, change));
            }

            let diff_str = diff_lines.join("\n");
            diff_items.push(DiffItem::new(&d, &diff_str));
        }

        diff_items
    }

    pub fn get_object_diffs(&self) -> Vec<String> { 
        let mut diffs = self.get_internal_object_diff();
        diffs.append(&mut self.get_internal_column_object_diff());
        diffs.sort();
        diffs.dedup();

        diffs
    }

    fn get_internal_object_diff(&self) -> Vec<String> {
        let mut missing_objects = vec![];
        for o in self.source.objects.iter() {
            if !self.target.objects.contains(o) {
                missing_objects.push(o.to_string());
            }
        }
        missing_objects
    }

    fn get_internal_column_object_diff(&self) -> Vec<String> {
        let mut missing_objects = vec![];

        for o in self.source.columns.iter() {
            if !self.target.columns.contains(o) {
                let base_object = o.split(".").next().expect("Missing base object").to_string();
                missing_objects.push(base_object);
            }
        }

        missing_objects
    }
}
