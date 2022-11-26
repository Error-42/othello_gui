# Othello gui specification

## v2.0.0-rc1

### 1st message: AI -> GUI

```
<version information>
```

`version information`: version of protocol, this must be `v1.0.0`

### 2nd message: GUI -> AI

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

### 3rd message AI -> GUI

```
<move>
```

`move`: consisting of a letter: columns a-h (left-to-right) and a number: rows 1-8 (top-to-bottom) 

### Example

AI -> GUI

```
v1.0.0
```

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

## v1.0.0

### 1st message: AI -> GUI

```
<version information>
```

`version information`: version of protocol, this must be `v1.0.0`

### 2nd message: GUI -> AI

```
<board (8 lines)>
<max time>
<n: number of possible moves> <move #1> <move #2> ... <move #n>
```

`board`: contains 8 lines, each line contains 8 character (not including `(\r)\n`) representing a tile.

- `.`: empty
- `X`: black
- `O`: white

`max time`: maximum time for computation in ms, will be whole number  
`move`: consisting of a letter: columns a-h (left-to-right) and a number: rows 1-8 (top-to-bottom)

### 3rd message AI -> GUI

```
<move>
```

`move`: consisting of a letter: columns a-h (left-to-right) and a number: rows 1-8 (top-to-bottom) 

### Example

AI -> GUI

```
v1.0.0
```

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
