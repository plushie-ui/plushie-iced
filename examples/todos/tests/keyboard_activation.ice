viewport: 500x800
mode: Immediate
preset: Empty
-----
click #new-task
type "Buy milk"
type enter
type "Walk the dog"
type enter
expect "2 tasks left"
click "Buy milk"
expect "1 task left"
type space
expect "2 tasks left"
type enter
expect "1 task left"
