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
    let active = taskwarrior::get_active_tasks()
        .expect("Active Tasks");
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
        .rows(tasks.rows)
        .selected_rows(active);
    
    let task_pane = Panel::new(
        OnEventView::new(
            tasks_table.with_name("tasks_table"))
            .on_event('a', show_add_task_dialog)
            .on_event('d', task_done)
            .on_event(cursive::event::Key::Del, task_delete)
            .on_event(cursive::event::Key::Backspace, task_delete)
            .on_event(cursive::event::Key::Enter, task_toggle)
            .on_event(' ', task_toggle)
        ).title("Tasks");
    
    let intervals_table = SimpleTableView::default()
        .columns(intervals_columns)
        .rows(intervals.rows);

    let interval_pane = Panel::new(
        OnEventView::new(
            intervals_table.with_name("intervals_table"))
            .on_event(cursive::event::Key::Del, time_delete)
            .on_event(cursive::event::Key::Backspace, time_delete)
        ).title("Intervals");

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
                    let focus_row = view.focus_row();
                    let tasks_columns: Vec<TableColumn> = tasks.columns
                        .into_iter()
                        .zip(tasks.colsizes)
                        .map(|(title, width)| TableColumn::new(title, Some(TableColumnWidth::Absolute(width))))
                        .collect();
                    let active = taskwarrior::get_active_tasks()
                        .expect("Active tasks");
    
                    view.set_columns(tasks_columns);
                    view.set_rows(tasks.rows);
                    view.set_selected_rows(active);
                    if focus_row.is_some() {
                        view.set_focus_row(focus_row.unwrap());
                    }
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
                let intervals = taskwarrior::get_interval_list(&mut text)
                    .expect("Intervals List");
            
                s.call_on_name("intervals_table", |view: &mut SimpleTableView| {
                    let focus_row = view.focus_row();
                    let intervals_columns: Vec<TableColumn> = intervals.columns
                        .into_iter()
                        .zip(intervals.colsizes)
                        .map(|(title, width)| TableColumn::new(title, Some(TableColumnWidth::Absolute(width))))
                        .collect();
                    view.set_columns(intervals_columns);
                    view.set_rows(intervals.rows);
                    if focus_row.is_some() {
                        view.set_focus_row(focus_row.unwrap());
                    }
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
                    .on_submit(cb_task_add)
                    .with_name("new_task_name")
                    .fixed_width(50)
            ).on_event(cursive::event::Key::Esc, cancel_dialog))
        .dismiss_button("Cancel")
    )
}

fn cb_task_add(s: &mut Cursive, text: &str) {
    taskwarrior::add_task(text)
        .expect("Add task");
    s.pop_layer();
}

fn task_toggle(s: &mut Cursive) {
    s.call_on_name("tasks_table", |view: &mut SimpleTableView| {
        let task_id = match view.focus_row() {
            Some(index) => index,
            None => {
                return ();
            }
        };

        let active = taskwarrior::get_active_tasks()
            .expect("Active tasks");
        active.iter()
            .for_each(|index| {
                taskwarrior::stop_task(&(index + 1).to_string())
                    .expect("Stop task");
            });
        
        if active.len() == 1 && active.contains(&task_id) {
            return;
        }

        taskwarrior::start_task(&(task_id + 1).to_string())
            .expect("Task start");
    });
}

fn task_delete(s: &mut Cursive) {
    s.add_layer(OnEventView::new(
        Dialog::text("Are you sure?")
            .button("Ok", cb_delete_task)
            .dismiss_button("Cancel"))
        .on_event(cursive::event::Key::Esc, cancel_dialog));
}

fn time_delete(s: &mut Cursive) {
    s.add_layer(OnEventView::new(
        Dialog::text("Are you sure?")
            .button("Ok", cb_delete_time)
            .dismiss_button("Cancel"))
        .on_event(cursive::event::Key::Esc, cancel_dialog));
}

fn cb_delete_task(s: &mut Cursive) {
    s.call_on_name("tasks_table", |view: &mut SimpleTableView| {
        match view.focus_row() {
            Some(index) => {
                let task_id = view.borrow_row(index)
                    .expect("Highlighted row")
                    .get(0)
                    .expect("Highlighted 0 cell");
                taskwarrior::delete_task(task_id)
                    .expect("Delete task");
            },
            None => {()}
        }
    });
    s.pop_layer();
}

fn cb_delete_time(s: &mut Cursive) {
    s.call_on_name("intervals_table", |view: &mut SimpleTableView| {
        match view.focus_row() {
            Some(index) => {
                let interval_id = view.borrow_row(index)
                    .expect("Highlighted row")
                    .get(3)
                    .expect("Highlighted 0 cell");
                taskwarrior::delete_time(interval_id)
                    .expect("Delete interval");
            },
            None => {()}
        }
    });
    s.pop_layer();
}

fn task_done(s: &mut Cursive) {
    s.call_on_name("tasks_table", |view: &mut SimpleTableView| {
        match view.focus_row() {
            Some(index) => {
                let task_id = view.borrow_row(index)
                    .expect("Highlighted row")
                    .get(0)
                    .expect("Highlighted 0 cell");
                taskwarrior::done_task(task_id)
                    .expect("Add task");
            },
            None => {()}
        }
    });
}

fn cancel_dialog(s: &mut Cursive) {
    s.pop_layer();
}

