use tui::App;

mod differ;
mod parser;
mod types;
mod tui;

fn main() {
    let source_sql = include_str!("../ddls/source.sql");
    let target_sql = include_str!("../ddls/target.sql");

    let source = parser::SqlParser::new(&source_sql);
    let target = parser::SqlParser::new(&target_sql);

    let differ = differ::Differ::new(source, target);
    let diff_items = differ.get_diff_items();

    color_eyre::install().unwrap();
    let terminal = ratatui::init();
    App::new(diff_items).run(terminal).unwrap();

    ratatui::restore();
}
