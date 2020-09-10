# Parrot grammar

This document describes the grammar of the **parrot** scripting language.

```text
script  -> command
command -> quit  | help   | filter
           clear | run    | show
           edit  | update | delete

quit    -> 'q' | 'quit'
help    -> 'h' | 'help'
clear   -> 'c' | 'clear'
edit    -> 'e' | 'edit'
filter  -> ('f' | 'filter') (name | tag | '~' | '+' | '-')
run     -> ('r' | 'run') '*'?
show    -> ('s' | 'show') '*'?
update  -> ('u' | 'update') '*'?
delete  -> ('d' | 'delete') '*'?

name    -> [A-Za-z-_]+
tag     -> '#' [A-Za-z-_]+
```

