# Othello gui specification

## GUI -> AI

```
<board (8 lines)>
<next player>
<max time>
<number of possible moves> <move #1> <move #2> ... <move #n>
```

`board`: contains 8 lines, each line contains 8 character (not including `(\r)\n`) representing a tile.

- `.`: empty
- `X`: black
- `O`: white

`next player`: pieces of player having the next move, same as board tiles
`max time`: maximum time for computation in ms, will be whole number  
`move`: consisting of a letter: columns a-h (left-to-right) and a number: rows 1-8 (top-to-bottom)

## AI -> GUI

```
<move>
<notes (optional)>
```

`move`: consisting of a letter: columns a-h (left-to-right) and a number: rows 1-8 (top-to-bottom) 
`notes`: additional info provided to display

## Example

GUI -> AI

```
........
........
...X....
...XX...
...XO...
........
........
........
O
3000
3 c3 e3 c5
```

AI -> GUI (within 3000 ms)

```
e3
```

State of the board after this:

```
........
........
...XO...
...XO...
...XO...
........
........
........
```
