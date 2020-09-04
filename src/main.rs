use std::thread;
use std::time::Duration;

use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Dialog, LinearLayout, Panel, EditView, OnEventView};

mod cursive_simple_table_view;
mod taskwarrior;

use cursive_simple_table_view::{SimpleTableView, TableColumn, TableColumnWidth};

fn main() {
    let mut tasks_text = String::new();
    let tasks = taskwarrior::get_task_list(&mut tasks_text)
        .expect("Task List");
    let tasks_columns: Vec<TableColumn> = tasks.columns
        .into_iter()
        .zip(tasks.colsizes)
        .map(|(title, width)| TableColumn::new(title, Some(TableColumnWidth::Absolute(width))))
        .collect();

    let mut intervals_text = String::new();
    let intervals = taskwarrior::get_interval_list(&mut intervals_text)
        .expect("Interval List");
    let intervals_columns: Vec<TableColumn> = intervals.columns
        .into_iter()
        .zip(intervals.colsizes)
        .map(|(title, width)| TableColumn::new(title, Some(TableColumnWidth::Absolute(width))))
        .collect();

    let mut siv = cursive::default();

    siv.add_global_callback(cursive::event::Key::Esc, |s : &mut Cursive| s.quit());
    siv.load_toml(include_str!("../assets/style.toml")).unwrap();

    let tasks_table = SimpleTableView::default()
        .columns(tasks_columns)
        .rows(tasks.rows);
    
    let task_pane = Panel::new(
        OnEventView::new(
            tasks_table.with_name("tasks_table"))
            .on_event('a', show_add_task_dialog)
            .on_event('d', task_done)
            .on_event(cursive::event::Key::Del, task_delete)
            .on_event(cursive::event::Key::Backspace, task_delete)
        ).title("Tasks");
    
    let intervals_table = SimpleTableView::default()
        .columns(intervals_columns)
        .rows(intervals.rows);

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
            cb_sink.send(Box::new(move |s: &mut Cursive| {
                let mut text = String::new();
                let tasks = taskwarrior::get_task_list(&mut text)
                    .expect("Task List");

                s.call_on_name("tasks_table", |view: &mut SimpleTableView| {
                    view.set_rows(tasks.rows);
                });
            })).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    let cb_sink = siv.cb_sink().clone();
    thread::spawn(move || {
        loop {
            cb_sink.send(Box::new(move |s: &mut Cursive| {
                let mut text = String::new();
                let tasks = taskwarrior::get_interval_list(&mut text)
                    .expect("Intervals List");

                s.call_on_name("intervals_table", |view: &mut SimpleTableView| {
                    view.set_rows(tasks.rows);
                });
            })).unwrap();
            thread::sleep(Duration::from_secs(1));
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
