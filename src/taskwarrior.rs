use std::error::Error;
use std::str;
use std::cmp;
use std::process::Command;

pub struct TaskList<'a> {
    pub colsizes: Vec<usize>,
    pub columns: Vec<&'a str>,
    pub rows: Vec<Vec<&'a str>>
}

pub fn get_active_tasks<'a>() -> Result<Vec<usize>, Box<dyn Error>> {
    let stdout = Command::new("task")
        .arg("active")
        .output()?
        .stdout;

    let text = String::from_utf8(stdout)?;

    let mut list = parse_task_list(&text, false)?;

    let indices: Vec<usize> = list.rows
        .iter_mut()
        .map(|row| {
            let index = *row.get(0).unwrap();
            index.parse::<usize>().unwrap() - 1
        })
        .collect();

    Ok(indices)
}

pub fn get_interval_list<'a, 'b>(text: &'a mut String)-> Result<TaskList<'a>, &'b str> {
    let output = Command::new("timew")
        .arg("summary")
        .arg(":ids")
        .output();

    let stdout = match output {
        Ok(o) => String::from_utf8(o.stdout).unwrap_or_default(),
        Err(_) => return Err("Could not convert command output to UTF-8 string")
    };

    text.push_str(&stdout);

    parse_task_list(text, false)
}

pub fn get_task_list<'a, 'b>(text: &'a mut String) -> Result<TaskList<'a>, &'b str> {
    let output = Command::new("task")
        .arg("next")
        .output();

    let stdout = match output {
        Ok(o) => String::from_utf8(o.stdout).unwrap_or_default(),
        Err(_) => return Err("Could not convert command output to UTF-8 string")
    };

    text.push_str(&stdout);

    parse_task_list(text, true)
}

pub fn add_task<'a, 'b>(text: &'a str) -> Result<(), &'b str>{
    let output = Command::new("task")
        .arg("add")
        .arg(text)
        .output();

    match output {
        Ok(_) => Ok(()),
        Err(_) => Err("Could not add task")
    }
}

pub fn delete_task<'a, 'b>(task_id: &'a str) -> Result<(), &'b str>{
    let output = Command::new("task")
        .arg("delete")
        .arg("rc.confirmation:no")
        .arg(task_id)
        .output();

    match output {
        Ok(_) => Ok(()),
        Err(_) => Err("Could not delete task")
    }
}

fn parse_task_list<'a, 'b>(text: &'a str, sort: bool) -> Result<TaskList<'a>, &'b str> {
    let mut lines = text.lines();
    if lines.count() < 3 {
        return Ok(TaskList {
            colsizes: [].to_vec(),
            columns: [].to_vec(),
            rows: [].to_vec()
        });
    }

    lines = text.lines();
    let colsizes: Vec<usize> = get_column_sizes(&mut lines);

    lines = text.lines();
    let first_line = lines.nth(1).unwrap_or_default();
    let columns = split_row(first_line, &colsizes);

    lines = text.lines();
    let mut rows: Vec<Vec<&str>> = lines
        .skip(3)
        .take_while(|x: &&str| (*x).trim().len() > 0)
        .map(|line: &str| split_row(line, &colsizes))
        .collect();

    if sort {
        rows.sort_by(|a: &Vec<&str>, b: &Vec<&str>| {
            let sa = *a.get(0).unwrap();
            let sb = *b.get(0).unwrap();
            let ia: u32 = sa.parse().unwrap();
            let ib: u32 = sb.parse().unwrap();

            ia.cmp(&ib)
        });
    }

    Ok(TaskList { colsizes, columns, rows })
}

fn split_row<'a>(text: &'a str, colsizes: &Vec<usize>) -> Vec<&'a str> {
    let max = text.len();
    let mut save: usize = 0;
    colsizes.iter()
        .map(|width| {
            let start = cmp::min(max, save);
            let end = cmp::min(max, start + width);
            save += width + 1;
            text[start..end].trim()
        })
        .collect()
}

fn get_column_sizes(lines: &mut std::str::Lines) -> Vec<usize> {
    lines
        .nth(2)
        .unwrap_or_default()
        .split_ascii_whitespace()
        .map(|x: &str| x.len())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_list() {
        let data = "
ID Description
-- -----------
1  Buy milk
3  Bake cake
2  Buy eggs

3 tasks.";

        let result = parse_task_list(data, true);
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
    fn test_parse_task_list_2() {
        let data = "
Wk  Date       Day ID Tags                       Start      End    Time   Total
--- ---------- --- -- ----------------------- -------- -------- ------- -------
W36 2020-09-03 Thu @6 Planning with GL and KS  9:00:00 10:00:00 1:00:00
                   @5 AlwaysOn activities     10:14:59 11:30:00 1:15:01
                   @4 AlwaysOn activities     12:00:00 13:08:36 1:08:36
                   @3 Backlog grooming        13:10:00 15:20:00 2:10:00
                   @2 code reviews            15:38:59 15:59:55 0:20:56
                   @1 code reviews            16:19:10 17:15:16 0:56:06 6:50:39
                                                                                
                                                                        6:50:39";

        let result = parse_task_list(data, false);
        assert_eq!(true, result.is_ok());

        let result = result.unwrap();
        assert_eq!(9, result.columns.len());
        assert_eq!(["Wk", "Date", "Day", "ID", "Tags", "Start", "End", "Time", "Total"].to_vec(), result.columns);

        assert_eq!(6, result.rows.len());
    }

    #[test]
    fn test_parse_task_list_empty() {
        let data = "No matches.";

        let result = parse_task_list(data, true );
        assert_eq!(true, result.is_ok());

        let result = result.unwrap();
        assert_eq!(0, result.columns.len());

        assert_eq!(0, result.rows.len());
    }

    #[test]
    fn column_sizes() {
        let data = "
ID Description Age
-- ----------- ---
1  Buy milk    1
2  Buy eggs    2
3  Bake cake   5

3 tasks.";

        let colsizes = get_column_sizes(&mut data.lines());
        assert_eq!([2, 11, 3].to_vec(), colsizes);
    }
    
    #[test]
    fn split_line() {
        let line = " 1 9w  Intercom adhoc    0.38";
        let colsizes = [2, 3, 17, 4].to_vec();

        let x = split_row(line, &colsizes);

        assert_eq!(4, x.len());
        assert_eq!("1", *x.get(0).unwrap());
        assert_eq!("9w", *x.get(1).unwrap());
        assert_eq!("Intercom adhoc", *x.get(2).unwrap());
        assert_eq!("0.38", *x.get(3).unwrap());
    }
}
