viewport: 500x800
mode: Immediate
preset: Empty
-----
click #new-task
type "Task one"
type enter
type "Task two"
type enter
type "Task three"
type enter
expect "3 tasks left"
type tab
type space
expect "2 tasks left"
type escape
type tab
type tab
type tab
type tab
type space
expect "1 task left"
