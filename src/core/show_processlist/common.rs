use crate::models::ShowProcesslistInfo;
use prettytable::{format, Cell, Row, Table};

pub fn get_infos_table(infos: &Vec<ShowProcesslistInfo>) -> String {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    // 设置title
    table.set_titles(Row::new(vec![
        Cell::new("Id"),
        Cell::new("User"),
        Cell::new("Host"),
        Cell::new("db"),
        Cell::new("Command"),
        Cell::new("Time"),
        Cell::new("State"),
        Cell::new("Info"),
    ]));

    for info in infos.iter() {
        let mut info_vec = Vec::new();
        info_vec.push(Cell::new(&info.id.clone().unwrap().to_string()));
        info_vec.push(Cell::new(
            info.user
                .clone()
                .or(Some(String::from("")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.host
                .clone()
                .or(Some(String::from("")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.db.clone().or(Some(String::from(""))).as_ref().unwrap(),
        ));
        info_vec.push(Cell::new(
            info.command
                .clone()
                .or(Some(String::from("")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(&info.time.clone().unwrap().to_string()));
        info_vec.push(Cell::new(
            info.state
                .clone()
                .or(Some(String::from("")))
                .as_ref()
                .unwrap(),
        ));
        info_vec.push(Cell::new(
            info.info
                .clone()
                .or(Some(String::from("")))
                .as_ref()
                .unwrap(),
        ));

        table.add_row(Row::new(info_vec));
    }

    table.to_string()
}
