use std::thread;
use std::time::Duration;

use rand::Rng;

// use chrono::Utc;

use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Dialog, LinearLayout, Panel, EditView, OnEventView};

mod cursive_simple_table_view;
mod taskwarrior;

use cursive_simple_table_view::{SimpleTableView, TableColumn, TableColumnWidth};

fn main() {
    let mut siv = Cursive::new(|| {
        let crossterm_backend = cursive::backends::crossterm::Backend::init().unwrap();
        let buffered_backend = cursive_buffered_backend::BufferedBackend::new(crossterm_backend);
        Box::new(buffered_backend)
    });

    siv.add_global_callback(cursive::event::Key::Esc, |s : &mut Cursive| s.quit());
    siv.load_toml(include_str!("../assets/style.toml")).unwrap();

    // let (columns, rows, _ ,_) = gen_data();
    // let tasks_table = SimpleTableView::default()
    //     .columns(columns)
    //     .rows(rows)
    //     .selected_row(Some(2));

    let mut text = String::new();
    let tasks = taskwarrior::get_task_list(&mut text).unwrap();
    let columns: Vec<TableColumn> = tasks.columns
        .into_iter()
        .zip(tasks.colsizes)
        .map(|(title, width)| TableColumn::new(title, Some(TableColumnWidth::Absolute(width))))
        .collect();

    let tasks_table = SimpleTableView::default()
        .columns(columns)
        .rows(tasks.rows);
    
    let task_pane = Panel::new(
        OnEventView::new(
            tasks_table.with_name("tasks_table"))
            .on_event('a', show_add_task_dialog)
            .on_event('d', task_done)
            .on_event(cursive::event::Key::Del, task_delete)
            .on_event(cursive::event::Key::Backspace, task_delete)
        ).title("Tasks");
    
    let (columns, rows, _ ,_) = gen_data();
    let intervals_table = SimpleTableView::default()
        .columns(columns)
        .rows(rows);

    let interval_pane = Panel::new(
        intervals_table
            .with_name("intervals_table"))
        .title("Intervals");

    let view = LinearLayout::horizontal()
        .child(task_pane.full_height().fixed_width(50))
        .child(interval_pane.full_height().full_width());

    siv.add_fullscreen_layer(view);

    let cb_sink = siv.cb_sink().clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            // let now = Utc::now().format("%H:%M:%S");
    
            cb_sink.send(Box::new(move |_s| {
                let (_columns, _rows, _num_cols, _num_rows) = gen_data();
                // s.call_on_name("tasks_table", |view: &mut SimpleTableView| {
                //     view.set_columns(columns);
                //     view.set_rows(rows);
                // });
                // s.call_on_name("interval_pane", |view: &mut TextView| {
                //     view.set_content(format!("{} {} {}", now, num_cols, num_rows));
                // });
            })).unwrap();
        }
    });

    siv.run();
}

fn show_add_task_dialog(s: &mut Cursive) {
    s.add_layer(Dialog::new()
        .title("Add Task")
        .content(
            OnEventView::new(
                EditView::new()
                    .filler(" ")
                    .on_submit(task_add)
                    .with_name("new_task_name")
                    .fixed_width(50)
            ).on_event(cursive::event::Key::Esc, cancel_dialog))
        .dismiss_button("Cancel")
    )
}

fn task_add(_s: &mut Cursive, _text: &str) {}

fn task_delete(s: &mut Cursive) {
    s.add_layer(OnEventView::new(
        Dialog::text("Are you sure?")
            .button("Ok", |s: &mut Cursive| {
                cancel_dialog(s);
            })
            .dismiss_button("Cancel"))
        .on_event(cursive::event::Key::Esc, cancel_dialog));
}

fn cancel_dialog(s: &mut Cursive) {
    s.pop_layer();
}

fn task_done(_s: &mut Cursive) {}

fn gen_data() -> (Vec<TableColumn>, Vec<Vec<String>>, usize, usize) {
    let mut rng = rand::thread_rng();

    let num_cols = rng.gen_range(3, 6);
    let mut cols = Vec::new();
    for i in 0..num_cols {
        let col = TableColumn::new(format!("C{}", i+1), None);
        cols.push(col);
    }

    let mut rows = Vec::new();
    let num_rows = rng.gen_range(4, 10);
    for i in 0..num_rows {
        let mut row = Vec::new();
        row.push(format!("Name {}", i));
        for _ in 1..num_cols {
            row.push(format!("{}", rng.gen_range(0, 255)));
        }
        rows.push(row);
    }

    (cols, rows, num_cols, num_rows)
}
