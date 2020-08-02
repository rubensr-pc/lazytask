package main

import (
	"github.com/gdamore/tcell"
	"github.com/rivo/tview"
)

func main() {
	app := tview.NewApplication()
	table := tview.NewTable().
		SetBorders(true).
		SetSelectable(true, false)
	twTasks := tview.NewBox()

	table.
		SetTitle("Tasks").
		SetBorder(true)

	flex := tview.NewFlex().
		AddItem(table, 0, 1, false).
		AddItem(twTasks.SetBorder(true).SetTitle("Right (20 cols)"), 0, 1, false)

	nextPane := func() {
		if app.GetFocus() == table {
			app.SetFocus(twTasks)
		} else {
			app.SetFocus(table)
		}
	}

	app.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		if event.Key() == tcell.KeyRight || event.Key() == tcell.KeyLeft {
			nextPane()
		}

		return event
	})

	if err := app.SetRoot(flex, true).SetFocus(table).Run(); err != nil {
		panic(err)
	}
}
