# sbar

sbar is modular, asynchronous and configurable bar for dwm written in rust.

# Features

- signal for updating each component
- yaml config file
- async
- per-component interval (and updating)

note: sbar is still in early development

### Installation

By cargo

    cargo install sbar


### Usage

    sbar [-c configpath] [-v] [h]

### Configuration

create `$HOME/.config/sbar/config.yaml`

see [default config](https://github.com/sleepntsheep/sbar/blob/main/src/config.rs#L1) as example

built-in modules:
- exec - execute a command and return the result
- memory - formatted memory
- battery - monitor battery (take battery index as param, default is 0)
- time - formatted time
- echo - return all params joined together as string 
  take params[0] as program name and params[1..len] as args
- sep - return seperator (defined in config.yaml)

property
- params is list of string
- interval is how often to update each component, in second
  not putting in interval use default which is never update


### Signal

put signal in wanted component's config 

    - name: echo
      params:
        - TEST
      signal:
        44

then to call it, do 

  kill -44 $(pidof sbar)

replace 44 with your signal

### Todo

- [x] signal support for updating, etc
- [x] per component update interval and signal
- [ ] make code not bad
