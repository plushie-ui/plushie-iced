viewport: 500x800
mode: Immediate
preset: Empty
-----
click #new-task
type "First task"
type enter
type "Second task"
type enter
expect "2 tasks left"
type tab
type space
expect "1 task left"
type tab
type space
expect "0 tasks left"
type tab
type space
expect "1 task left"
type tab
type space
expect "2 tasks left"
