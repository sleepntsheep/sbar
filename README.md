# sbar

sbar is modular, asynchronous and configurable (yaml config file) bar for dwm written in rust.

note: sbar is still in early development

### Installation

By cargo

    cargo install sbar


### Usage

    sbar [-c configpath] [-v] [h]

### Configuration

create `$HOME/.config/sbar/config.yml`

    list:
    - name: exec
        params:
        - echo
        - "Welcome sheep"
    - name: memory
        params:
        - LMAO
    - name: time
    sep: " | "
    delay: 1000

built-in modules:
- exec
- memory
- time

params is list of string

##### Exec
take params[0] as program name and params[1..len] as args

### Todo
- [] signal support for updating, etc
