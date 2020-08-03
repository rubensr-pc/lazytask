package main

import (
	"fmt"
	"log"
	"os/exec"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/gdamore/tcell"
	"github.com/rivo/tview"
)

const refreshInterval = 1000 * time.Millisecond

var noActiveTaskRegex = regexp.MustCompile(`No matches`)
var app *tview.Application

func contains(s []int, e int) bool {
	for _, a := range s {
		if a == e {
			return true
		}
	}
	return false
}

func modal(p tview.Primitive, width, height int) tview.Primitive {
	return tview.NewFlex().
		AddItem(nil, 0, 1, false).
		AddItem(tview.NewFlex().SetDirection(tview.FlexRow).
			AddItem(nil, 0, 1, false).
			AddItem(p, height, 1, false).
			AddItem(nil, 0, 1, false), width, 1, false).
		AddItem(nil, 0, 1, false)
}

func getActiveTasks() []int {
	out, err := exec.Command("task", "active").Output()
	if err != nil {
		return nil
	}
	src := fmt.Sprintf("%s", out)

	var result []int
	allRows := strings.Split(src, "\n")
	var colSizes []int
	for _, v := range strings.Split(allRows[2], " ") {
		colSizes = append(colSizes, len(v))
	}

	for _, data := range allRows[3 : len(allRows)-3] {
		i, err := strconv.Atoi(strings.Trim(data[0:colSizes[0]], " "))
		if err != nil {
			log.Fatal(err)
		}
		result = append(result, i)
	}

	return result
}

func stopTasks(t []int) {
	if t == nil {
		return
	}

	valuesText := []string{}

	// Create a string slice using strconv.Itoa.
	// ... Append strings to it.
	for i := range t {
		number := t[i]
		text := strconv.Itoa(number)
		valuesText = append(valuesText, text)
	}

	// Join our string slice.
	result := strings.Join(valuesText, ",")

	_, err := exec.Command("task", "stop", result).Output()
	if err != nil {
		log.Fatal(err)
	}
}

func startTask(t string) {
	_, err := exec.Command("task", "start", t).Output()
	if err != nil {
		log.Fatal(err)
	}
}

func addTask(t string) {
	_, err := exec.Command("task", "add", t).Output()
	if err != nil {
		log.Fatal(err)
	}
}

func doneTask(t string) {
	_, err := exec.Command("task", "done", t).Output()
	if err != nil {
		log.Fatal(err)
	}
}

func main() {
	var pages *tview.Pages

	app = tview.NewApplication()
	tasksTable := tview.NewTable()

	tasksTable.
		SetSelectedFunc(func(row int, column int) {
			activeTasks := getActiveTasks()
			newTask := tasksTable.GetCell(row, 0).Text
			stopTasks(activeTasks)
			if len(activeTasks) == 1 && strconv.Itoa(activeTasks[0]) == newTask {
				return
			}
			startTask(newTask)
		})

	tasksTable.
		SetFixed(1, 1).
		SetBorders(false).
		SetSeparator(tview.Borders.Vertical).
		SetSelectable(true, false).
		SetTitle("Tasks").
		SetBorder(true)

	intervalsTable := tview.NewTable()

	intervalsTable.
		SetFixed(1, 1).
		SetBorders(false).
		SetSeparator(tview.Borders.Vertical).
		SetSelectable(true, true).
		SetTitle("Day").
		SetBorder(true)

	flex := tview.NewFlex().
		AddItem(tasksTable, 0, 1, false).
		AddItem(intervalsTable, 0, 2, false)

	addTaskField := tview.NewInputField()
	addTaskModal := tview.NewForm().
		AddFormItem(addTaskField.
			SetFieldWidth(0).
			SetDoneFunc(func(key tcell.Key) {
				switch key {
				case tcell.KeyEnter:
					newTaskName := strings.Trim(addTaskField.GetText(), " ")
					if newTaskName != "" {
						addTaskField.SetText("")
						pages.HidePage("addtask")
						app.SetFocus(tasksTable)
						addTask(newTaskName)
					}
				}
			})).
		SetCancelFunc(func() {
			addTaskField.SetText("")
			pages.HidePage("addtask")
			app.SetFocus(tasksTable)
		})

	addTaskModal.
		SetBorder(true).
		SetTitle("Add Task")

	pages = tview.NewPages().
		AddPage("main", flex, true, true).
		AddPage("addtask", modal(addTaskModal, 40, 5), true, false)

	nextPane := func() {
		if app.GetFocus() == tasksTable {
			app.SetFocus(intervalsTable)
		} else {
			app.SetFocus(tasksTable)
		}
	}

	app.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		frontPage, _ := pages.GetFrontPage()
		if frontPage != "main" {
			return event
		}

		switch key := event.Key(); key {
		case tcell.KeyRight, tcell.KeyLeft:
			nextPane()
		case tcell.KeyEsc:
			app.Stop()
		}

		return event
	})

	tasksTable.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		switch key := event.Key(); key {
		case tcell.KeyRune:
			switch rune := event.Rune(); rune {
			case 'a':
				pages.ShowPage("addtask")
				app.SetFocus(addTaskModal)
			case 'd':
				r, _ := tasksTable.GetSelection()
				t := tasksTable.GetCell(r, 0).Text
				doneTask(t)
			}
		}

		return event
	})

	intervalsTable.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		switch key := event.Key(); key {
		case tcell.KeyLeft, tcell.KeyRight:
			return nil
		}

		return event
	})

	go func() {
		for {
			time.Sleep(refreshInterval)
			out, err := exec.Command("timew", "summary", ":ids").Output()
			if err != nil {
				log.Fatal(err)
			}
			src := fmt.Sprintf("%s", out)
			allRows := strings.Split(src, "\n")
			var colSizes []int
			for _, v := range strings.Split(allRows[2], " ") {
				colSizes = append(colSizes, len(v))
			}

			if len(colSizes) != intervalsTable.GetColumnCount() {
				intervalsTable.Clear()
			}

			curCol := 0
			for i := range colSizes {
				newCol := curCol + colSizes[i]
				if newCol > len(allRows[1]) {
					newCol = len(allRows[1])
				}
				intervalsTable.SetCell(0, i,
					tview.NewTableCell(allRows[1][curCol:newCol]).
						SetTextColor(tcell.ColorTeal).
						SetSelectable(false))
				curCol = newCol + 1
			}

			for row, data := range allRows[3 : len(allRows)-4] {
				curCol = 0
				rowTextColor := tcell.ColorWhite
				for col := range colSizes {
					newCol := curCol + colSizes[col]
					if newCol > len(data) {
						newCol = len(data)
					}
					text := ""
					if curCol <= newCol {
						text = strings.Trim(data[curCol:newCol], " ")
					}
					intervalsTable.SetCell(row+1, col,
						tview.NewTableCell(text).
							SetTextColor(rowTextColor).
							SetSelectable(text != "" && col == 3))
					curCol = newCol + 1
				}
			}
		}
	}()

	go func() {
		for {
			time.Sleep(refreshInterval)

			activeTasks := getActiveTasks()

			out, err := exec.Command("task", "next").Output()
			if err != nil {
				log.Fatal(err)
			}
			src := fmt.Sprintf("%s", out)
			allRows := strings.Split(src, "\n")
			var colSizes []int
			for _, v := range strings.Split(allRows[2], " ") {
				colSizes = append(colSizes, len(v))
			}

			if len(colSizes) != tasksTable.GetColumnCount() || len(activeTasks) != tasksTable.GetRowCount()+1 {
				tasksTable.Clear()
			}

			curCol := 0
			for i := range colSizes {
				newCol := curCol + colSizes[i]
				if newCol > len(allRows[1]) {
					newCol = len(allRows[1])
				}
				tasksTable.SetCell(0, i,
					tview.NewTableCell(strings.Trim(allRows[1][curCol:newCol], " ")).
						SetTextColor(tcell.ColorTeal).
						SetSelectable(false))
				curCol = newCol + 1
			}

			sortedRows := allRows[3 : len(allRows)-3]
			sort.Strings(sortedRows)
			for row, data := range sortedRows {
				curCol = 0
				rowTextColor := tcell.ColorWhite
				if contains(activeTasks, row+1) {
					rowTextColor = tcell.ColorGreen
				}
				for col := range colSizes {
					newCol := curCol + colSizes[col]
					if newCol > len(data) {
						newCol = len(data)
					}
					tasksTable.SetCell(row+1, col,
						tview.NewTableCell(strings.Trim(data[curCol:newCol], " ")).
							SetTextColor(rowTextColor))
					curCol = newCol + 1
				}
			}
		}
	}()

	if err := app.SetRoot(pages, true).SetFocus(tasksTable).Run(); err != nil {
		panic(err)
	}
}
