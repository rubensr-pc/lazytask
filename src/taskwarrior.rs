use std::str;
use std::cmp;
use std::process::Command;

#[allow(dead_code)]
pub struct TaskList<'a> {
    pub columns: Vec<&'a str>,
    pub rows: Vec<Vec<&'a str>>
}

#[allow(dead_code)]
pub fn get_task_list<'a, 'b>(text: &'a mut String) -> Result<TaskList<'a>, &'b str> {
    let output = Command::new("task")
        .arg("next")
        .output();

    let stdout = match output {
        Ok(o) => String::from_utf8(o.stdout).unwrap_or_default(),
        Err(_) => return Err("Could not convert command output to UTF-8 string")
    };

    text.push_str(&stdout);

    parse_task_list(text)
}

fn parse_task_list<'a, 'b>(text: &'a str) -> Result<TaskList<'a>, &'b str> {
    let mut lines = text.lines();
    if lines.count() < 3 {
        return Ok(TaskList {
            columns: [].to_vec(),
            rows: [].to_vec()
        });
    }

    lines = text.lines();
    let colsizes: Vec<usize> = get_column_sizes(&mut lines);

    lines = text.lines();
    let first_line = lines.nth(0).unwrap_or_default();
    let columns = split_row(first_line, &colsizes);

    lines = text.lines();
    let mut rows: Vec<Vec<&str>> = lines
        .skip(2)
        .take_while(|x: &&str| (*x).trim().len() > 0)
        .map(|line: &str| split_row(line, &colsizes))
        .collect();

    rows.sort_by(|a: &Vec<&str>, b: &Vec<&str>| {
        let sa = *a.get(0).unwrap();
        let sb = *b.get(0).unwrap();
        let ia: u32 = sa.parse().unwrap();
        let ib: u32 = sb.parse().unwrap();

        ia.cmp(&ib)
    });
    Ok(TaskList { columns, rows })
}

fn split_row<'a>(text: &'a str, colsizes: &Vec<usize>) -> Vec<&'a str> {
    let max = text.len();
    colsizes.iter()
        .zip(colsizes.iter().skip(1))
        .map(|(a, b): (&usize, &usize)| {
            let end = cmp::min(max, *a + *b + 1);
            let start = *a;
            text[start..end].trim()
        })
        .collect()
}

fn get_column_sizes(lines: &mut std::str::Lines) -> Vec<usize> {
    let mut colsizes: Vec<usize> = lines
        .nth(1)
        .unwrap_or_default()
        .split_ascii_whitespace()
        .map(|x: &str| x.len())
        .collect();

    colsizes.insert(0, 0);

    colsizes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_list() {
        let data = "ID Description
-- -----------
1  Buy milk
3  Bake cake
2  Buy eggs

3 tasks.";

        let result = parse_task_list(data);
        assert_eq!(true, result.is_ok());

        let result = result.unwrap();
        assert_eq!(2, result.columns.len());
        assert_eq!(["ID", "Description"].to_vec(), result.columns);

        assert_eq!(3, result.rows.len());
        assert_eq!(["1", "Buy milk"].to_vec(), *result.rows.get(0).unwrap());
        assert_eq!(["2", "Buy eggs"].to_vec(), *result.rows.get(1).unwrap());
        assert_eq!(["3", "Bake cake"].to_vec(), *result.rows.get(2).unwrap());
    }

    #[test]
    fn test_parse_task_list_empty() {
        let data = "No matches.";

        let result = parse_task_list(data);
        assert_eq!(true, result.is_ok());

        let result = result.unwrap();
        assert_eq!(0, result.columns.len());

        assert_eq!(0, result.rows.len());
    }

    #[test]
    fn column_sizes() {
        let data = "ID Description
-- -----------
1  Buy milk
2  Buy eggs
3  Bake cake

3 tasks.";

        let colsizes = get_column_sizes(&mut data.lines());
        assert_eq!([0, 2, 11].to_vec(), colsizes);
    }
    
    #[test]
    fn split_line() {
        let line = "1  Buy milk";
        let colsizes = [0, 2, 20].to_vec();

        let x = split_row(line, &colsizes);

        assert_eq!(2, x.len());
        assert_eq!("1", *x.get(0).unwrap());
        assert_eq!("Buy milk", *x.get(1).unwrap());
    }
}
