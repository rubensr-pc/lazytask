use std::cmp;

use cursive::vec::Vec2;
use cursive::align::HAlign;
use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key};
use cursive::theme;
use cursive::view::{ScrollBase, View};
use cursive::With;
use cursive::Printer;

pub struct SimpleTableView {
    enabled: bool,
    scrollbase: ScrollBase,
    last_size: Vec2,
    compact: bool,

    columns: Vec<TableColumn>,
    rows: Vec<Vec<String>>,
    focus: usize,
    selected_row: Option<usize>
}

impl Default for SimpleTableView {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleTableView {
    pub fn new() -> Self {
        Self {
            enabled: true,
            scrollbase: ScrollBase::new(),
            last_size: Vec2::new(0, 0),
            compact: true,

            columns: Vec::new(),
            rows: Vec::new(),
            focus: 0,
            selected_row: None
        }
    }

    pub fn clear(&mut self) {
        self.rows.clear();
        self.selected_row = None;
        self.focus = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn set_selected_row(&mut self, row_index: Option<usize>) {
        self.selected_row = row_index;
    }

    pub fn selected_row(self: SimpleTableView, row_index: Option<usize>) -> Self {
        self.with(|t| t.set_selected_row(row_index))
    }

    pub fn set_focus_row(&mut self, row_index: usize) {
        self.focus = row_index;
        self.scrollbase.scroll_to(row_index);
    }

    pub fn set_columns(&mut self, columns: Vec<TableColumn>) {
        self.columns = columns;
        self.clear();

        self.last_size = Vec2::new(0, 0);
    }

    pub fn columns(self: SimpleTableView, columns: Vec<TableColumn>) -> Self {
        self.with(|t| t.set_columns(columns))
    }

    pub fn set_rows<S: Into<String>>(&mut self, rows: Vec<Vec<S>>) {
        let rows = rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|c| c.into())
                    .collect()
            })
            .collect();
        self.rows = rows;
        self.selected_row = None;
        self.focus = 0;
        self.scrollbase
            .set_heights(self.last_size.y.saturating_sub(2), self.rows.len());
        
        self.set_focus_row(0);
    }

    pub fn rows<S: Into<String>>(self: SimpleTableView, rows: Vec<Vec<S>>) -> Self {
        self.with(|t| t.set_rows(rows))
    }

    fn draw_columns<C: Fn(&Printer, &TableColumn, usize)>(
        &self,
        printer: &Printer,
        sep: &str,
        callback: C,       
    ) {
        let mut column_offset = 0;
        let column_count = self.columns.len();
        for (index, column) in self.columns.iter().enumerate() {
            let printer = &printer.offset((column_offset, 0)).focused(true);

            callback(printer, column, index);

            if index < column_count - 1 {
                printer.print((column.width + 1, 0), sep);
            }

            column_offset += column.width + 3;
        }
    }

    fn draw_item(&self, printer: &Printer, row_index: usize) {
        self.draw_columns(printer, "│ ", |printer, column, column_index| {
            let value = &self.rows[row_index][column_index];
            column.draw_row(printer, value);
        });
    }

    fn focus_up(&mut self, n: usize) {
        self.focus -= cmp::min(self.focus, n);
    }

    fn focus_down(&mut self, n: usize) {
        self.focus = cmp::min(self.focus + n, self.rows.len() - 1);
    }
}

impl View for SimpleTableView {
    fn draw(&self, printer: &Printer) {
        self.draw_columns(printer, "│ ", |printer, column, _| {
            let color = theme::ColorStyle::title_primary();

            printer.with_color(color, |printer| {
                column.draw_header(printer);
            });
        });

        if !self.compact {
            self.draw_columns(
                &printer.offset((0, 1)).focused(true),
                "┴─",
                |printer, column, _| {
                    printer.print_hline((0, 0), column.width + 1, "─");
                },
            );
        }
        let printer = &printer.offset((0, if self.compact { 1 } else { 2 })).focused(true);

        self.scrollbase.draw(printer, |printer, i| {
            let style = if i == self.focus && self.enabled {
                if printer.focused {
                    theme::Style::from(theme::ColorStyle::highlight())
                } else {
                    theme::Style::from(theme::ColorStyle::primary())
                }
            } else {
                match self.selected_row {
                    Some(r) => if r == i { 
                        theme::Style::from(theme::ColorStyle::highlight()).combine(theme::Effect::Reverse)
                    } else {
                        theme::Style::from(theme::ColorStyle::primary())
                    }
                    None => theme::Style::from(theme::ColorStyle::primary())
                }
            };

            if i < self.rows.len() {
                printer.with_style(style, |printer| {
                    self.draw_item(printer, i);
                });
            }
        });

        // Extend the vertical bars to the end of the view
        for y in self.scrollbase.content_height..printer.size.y {
            self.draw_columns(&printer.offset((0, y)), "│ ", |_, _, _| ());
        }
    }

    fn layout(&mut self, size: Vec2) {
        if size == self.last_size {
            return;
        }

        let item_count = self.rows.len();
        let column_count = self.columns.len();

        // Split up all columns into sized / unsized groups
        let (mut sized, mut usized): (Vec<&mut TableColumn>, Vec<&mut TableColumn>) = self
            .columns
            .iter_mut()
            .partition(|c| c.requested_width.is_some());

        // Subtract one for the separators between our columns (that's column_count - 1)
        let mut available_width = size.x.saturating_sub(column_count.saturating_sub(1) * 3);

        // Reduce the width in case we are displaying a scrollbar
        if size.y.saturating_sub(1) < item_count {
            available_width = available_width.saturating_sub(2);
        }

        // Calculate widths for all requested columns
        let mut remaining_width = available_width;
        for column in &mut sized {
            column.width = match *column.requested_width.as_ref().unwrap() {
                TableColumnWidth::Percent(width) => cmp::min(
                    (size.x as f32 / 100.0 * width as f32).ceil() as usize,
                    remaining_width,
                ),
                TableColumnWidth::Absolute(width) => width,
            };
            remaining_width = remaining_width.saturating_sub(column.width);
        }

        // Spread the remaining with across the unsized columns
        let remaining_columns = usized.len();
        for column in &mut usized {
            column.width = (remaining_width as f32 / remaining_columns as f32).floor() as usize;
        }

        self.scrollbase
            .set_heights(size.y.saturating_sub(2), item_count);
        self.last_size = size;
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        self.enabled
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if !self.enabled {
            return EventResult::Ignored;
        }

        let last_focus = self.focus;
        match event {
            Event::Key(Key::Up) => self.focus_up(1),
            Event::Key(Key::Down) => self.focus_down(1),
            Event::Key(Key::PageUp) => self.focus_up(10),
            Event::Key(Key::PageDown) => self.focus_down(10),
            Event::Key(Key::Home) => self.focus = 0,
            Event::Key(Key::End) => self.focus = self.rows.len() - 1,
            _ => return EventResult::Ignored
        }

        let focus = self.focus;
        self.scrollbase.scroll_to(focus);
        if !self.is_empty() && last_focus != focus {
            EventResult::Consumed(None)
        } else {
            EventResult::Ignored
        }
    }
}

#[allow(dead_code)]
pub enum TableColumnWidth {
    Percent(usize),
    Absolute(usize),
}

pub struct TableColumn {
    title: String,
    alignment: HAlign,
    width: usize,
    requested_width: Option<TableColumnWidth>
}

impl TableColumn {
    pub fn new<S: Into<String>>(title: S, requested_width: Option<TableColumnWidth>) -> Self {
        Self {
            title: title.into(),
            alignment: HAlign::Left,
            width: 0,
            requested_width
        }
    }

    fn draw_row(&self, printer: &Printer, value: &str) {
        let value = match self.alignment {
            HAlign::Left => format!("{:<width$} ", value, width = self.width),
            HAlign::Right => format!("{:>width$} ", value, width = self.width),
            HAlign::Center => format!("{:^width$} ", value, width = self.width),
        };

        printer.print((0, 0), value.as_str());
    }

    fn draw_header(&self, printer: &Printer) {
        let header = match self.alignment {
            HAlign::Left => format!(
                "{:<width$}",
                self.title,
                width = self.width
            ),
            HAlign::Right => format!(
                "{:>width$}",
                self.title,
                width = self.width
            ),
            HAlign::Center => format!(
                "{:^width$}",
                self.title,
                width = self.width
            ),
        };

        printer.print((0, 0), header.as_str());
    }
}