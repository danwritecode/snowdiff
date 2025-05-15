use std::collections::HashMap;

use indexmap::IndexSet;
use regex::Regex;
use sqlparser::ast::ObjectNamePart;
use sqlparser::dialect::SnowflakeDialect;
use sqlparser::{
    ast,
    parser::Parser
};


pub struct SqlParser {
    pub ddls: Vec<String>,
    pub ast: Vec<ast::Statement>,
    pub objects: IndexSet<String>,
    pub columns: IndexSet<String>,
}

impl SqlParser {
    pub fn new(sql: &str) -> Self {
        let sql = Self::preprocess_sql(sql);
        let ddls = sql.split(";").map(|s| s.to_string()).collect();

        let dialect = SnowflakeDialect {};
        let ast = Parser::parse_sql(&dialect, &sql).unwrap();

        let objects = Self::create_object_hashset(&ast);
        let columns = Self::create_object_column_hashset(&ast);

        Self { ddls, ast, objects, columns }
    }

    pub fn get_ddl_by_object(&self, object_name: &str) -> Option<String> {
        let object_index = self.objects.iter().position(|o| o == object_name)?;
        Some(self.ddls[object_index].clone())
    }

    fn preprocess_sql(sql: &str) -> String {
        // update timestamps causing errors
        let sql = Regex::new(r"TIMESTAMP_NTZ").unwrap().replace_all(&sql, "TIMESTAMPNTZ");
        let sql = Regex::new(r"TIMESTAMP_LTZ").unwrap().replace_all(&sql, "TIMESTAMPLTZ");
        let sql = Regex::new(r"TIMESTAMP_TZ").unwrap().replace_all(&sql, "TIMESTAMPTZ");

        // remove create or replace statements that are unsupported
        let sql = Regex::new(r"(?is)create\s+or\s+replace\s+(schema|database|task|procedure)\b.*?;").unwrap().replace_all(&sql, "");

        sql.to_string()
    }

    fn create_object_hashset(ast: &Vec<ast::Statement>) -> IndexSet<String> {
        let mut tables_views_hashset = IndexSet::new();

        for s in ast.iter() {
            match s {
                ast::Statement::CreateTable(ct) => {
                    let object_name = Self::create_joined_objectname(&ct.name.0);
                    tables_views_hashset.insert(object_name);
                },
                ast::Statement::CreateView { name, columns: _, ..} => {
                    let object_name = Self::create_joined_objectname(&name.0);
                    tables_views_hashset.insert(object_name);
                },
                _ => ()
            }
        }

        tables_views_hashset
    }

    fn create_object_column_hashset(ast: &Vec<ast::Statement>) -> IndexSet<String> {
        let mut tables_views_hashset = IndexSet::new();

        for s in ast.iter() {
            match s {
                ast::Statement::CreateTable(ct) => {
                    let object_name = Self::create_joined_objectname(&ct.name.0);
                    let columns = &ct.columns;

                    let columns = columns.iter().map(|c| {
                        let name = &c.name.value;
                        let data_type = &c.data_type.to_string();
                        format!("{}-{}", name, data_type)
                    });

                    for c in columns {
                        let object_column = format!("{}.{}", object_name, c);
                        tables_views_hashset.insert(object_column);
                    }
                },
                ast::Statement::CreateView { name, columns, ..} => {
                    let object_name = Self::create_joined_objectname(&name.0);
                    let columns = columns.iter().map(|c| {
                        let name = &c.name.value;
                        let data_type = &c.data_type.as_ref().map(|dt| {
                            dt.to_string()
                        }).unwrap_or(String::new());

                        format!("{}-{}", name, data_type)
                    });

                    for c in columns {
                        let object_column = format!("{}.{}", object_name, c);
                        tables_views_hashset.insert(object_column);
                    }
                },
                _ => ()
            }
        }

        tables_views_hashset
    }

    fn create_joined_objectname(name: &Vec<ObjectNamePart>) -> String {
        name
            .iter()
            .enumerate()
            .fold("".to_string(), |acc, (i, x)| {
                let name = match i {
                    0 => {
                        let name = &Self::normalize_db_name(&x.as_ident().unwrap().value);
                        format!("{}", name)
                    },
                    _ => {
                        let name = &x.as_ident().unwrap().value;
                        format!(".{}", name)
                    }
                };

                acc + &name
            })
    }

    fn normalize_db_name(name: &str) -> String {
        let name = name.to_lowercase();
        let names = HashMap::from([
            ("", ""),
        ]);

        match names.get(name.as_str()) {
            Some(n) => return n.to_string(),
            None => {
                // eprintln!("Missing db name: {}", name);
                name.to_string()
            }
        }
    }
}
