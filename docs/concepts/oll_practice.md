---
tags:
  - rubiks_cube
  - OLL
---
# OLL — Full Case Reference

## No Edges Flipped (Dot)

### Case #01 — Tie Fighter

- **Tags:** * dot
- **Algorithm:** `R U (x') U' R U l' R' U' l' U l F'`
- **Pattern:** LTR / LXR / LBR
- **Result:** Case #02 rotated 180°

### Case #02 — Dot Wall

- **Tags:** * dot
- **Algorithm:** `F R U R' U' F' f R U R' U' f'`
- **Pattern:** LTT / LXR / LBB
- **Result:** Case #02 rotated 180°

### Case #03 — Left Pirate

- **Tags:** * dot

- **Algorithm:** `r' R2 U R' U r U2 r' U R' r`
- **Pattern:** TTR / LXR / XBB
- **Result:** Case #04

### Case #04 — Right Pirate

- **Tags:** * dot
- **Algorithm:** `r' R U' r U2 r' U' R U' R2' r`
- **Pattern:** LTT / LXR / BBX
- **Result:** Case #03

### Case #05 — Rocky

- **Tags:** * dot
- **Algorithm:** `r' R U R U R' U' r R2' F R F'`
- **Pattern:** XTX / LXR / LBR
- **Result:** Case #07 rotated 180°

### Case #06 — Storage Unit

- **Tags:** * dot
- **Algorithm:** `F R U R' U (y') R' U2 R' F R F'`
- **Pattern:** TTT / LXR / XBX
- **Result:** Case #07 rotated clockwise 90°

### Case #07 — Diagonal

- **Tags:** * dot
- **Algorithm:** `R U R' U R' F R F' U2 R' F R F'`
- **Pattern:** XTT / LXR / LBX
- **Result:** Case #06 rotated 180°

### Case #30 — Checkers

- **Tags:** * dot
- **Algorithm:** `r' R U R U R' U' r2 R2' U R U' r'`
- **Pattern:** XTX / LXR / XBX
- **Result:** Case #30 (same)

## T-Shapes

### Case #47 — Overalls

- **Tags:** - T_shapes
- **Algorithm:** `F R U R' U' F'`
- **Pattern:** LTX / XXX / LBX
- **Result:** Case #37

### Case #48 — Slacks

- **Tags:** - T_shapes
- **Algorithm:** `R U R' U' R' F R F'`
- **Pattern:** TTX / XXX / BBX
- **Result:** Case #32

## C-Shapes

### Case #45 — Magnet

- **Tags:** - C_shapes
- **Algorithm:** `R' U' l' U l F' U R`
- **Pattern:** XXR / LXR / XXR
- **Result:** Case #38

### Case #46 — Metroid

- **Tags:** - C_shapes
- **Algorithm:** `R U R' U' (x) D' R' U R U' D (x')`
- **Pattern:** LTR / XXX / XBX
- **Result:** Case #32

## Squares

### Case #20 — Right Dragon

- **Tags:** L squares
- **Algorithm:** `r U2 R' U' R U' r'`
- **Pattern:** LXX / LXX / BBR
- **Result:** Case #22

### Case #21 — Left Dragon

- **Tags:** L squares
- **Algorithm:** `l' U2 L U L' U l`
- **Pattern:** XXR / XXR / LBB
- **Result:** Case #23

## Lightning Bolts

### Case #22 — Tetris Right

- **Tags:** L lightning_bolts
- **Algorithm:** `r U R' U R U2 r'`
- **Pattern:** TXR / XXR / XBB
- **Result:** Case #20

### Case #23 — Tetris Left

- **Tags:** L lightning_bolts
- **Algorithm:** `l' U' L U' L' U2 l`
- **Pattern:** LXT / LXX / BBX
- **Result:** Case #21

### Case #24 — Tetris Top


- **Tags:** L lightning_bolts
- **Algorithm:** `r R2' U' R U' R' U2 R U' R r'`
- **Pattern:** XXT / LXX / BBR
- **Result:** Case #21 rotated clockwise 90°

### Case #25 — Tetris Bottom

- **Tags:** L lightning_bolts
- **Algorithm:** `r' R2 U R' U R U2 R' U R' r`
- **Pattern:** TTR / LXX / XXB
- **Result:** Case #20 rotated clockwise 90°

### Case #43 — Right Pipe


- **Tags:** - lightning_bolts
- **Algorithm:** `R' F R U R' U' F' U R`
- **Pattern:** XTT / XXX / LBX
- **Result:** Case #36

### Case #44 — Left Pipe

- **Tags:** - lightning_bolts
- **Algorithm:** `L F' L' U' L U F U' L'`
- **Pattern:** TTX / XXX / XBR
- **Result:** Case #35 rotated 180°

## I-Shapes

### Case #08 — Caterpillar

- **Tags:** - I_shapes
- **Algorithm:** `R' U2 R2 U R' U R U2 (x') U' R' U (x)`
- **Pattern:** LXR / LXR / LXR
- **Result:** Case #13 rotated 180°

### Case #09 — Fridge

- **Tags:** - I_shapes
- **Algorithm:** `R' U' R U' R' d R' U R B`
- **Pattern:** TXR / LXR / BXR
- **Result:** Case #09 rotated clockwise 90°

### Case #10 — Ant at Wall

- **Tags:** - I_shapes
- **Algorithm:** `f R U R' U' R U R' U' f'`
- **Pattern:** LTT / XXX / LBB
- **Result:** Case #12 rotated 180°

### Case #11 — Butterfly

- **Tags:** - I_shapes
- **Algorithm:** `r' U' r U' R' U R U' R' U R r' U r`
- **Pattern:** LTR / XXX / LBR
- **Result:** Case #16 rotated clockwise 90°

## P-Shapes

### Case #35 — d Dot

- **Tags:** L P_shapes
- **Algorithm:** `R U B' U' R' U l U l'`
- **Pattern:** TTX / LXX / BXX
- **Result:** Case #43 rotated 180°

### Case #36 — q Dot

- **Tags:** L P_shapes
- **Algorithm:** `R' U' F U R U' R' F' R`
- **Pattern:** TXX / LXX / BBX
- **Result:** Case #43

### Case #37 — p Wall

- **Tags:** L P_shapes
- **Algorithm:** `F U R U' R' F'`
- **Pattern:** XXR / XXR / XBR
- **Result:** Case #47

### Case #38 — q Wall

- **Tags:** L P_shapes
- **Algorithm:** `F' U' L' U L F`
- **Pattern:** LXX / LXX / LBX
- **Result:** Case #47 rotated 180°

## Small L-Shapes

### Case #12

- **Tags:** L small_L
- **Algorithm:** `F R U R' U' R U R' U' F'`
- **Pattern:** LXT / XXR / LBB
- **Result:** Case #10 rotated 180°

### Case #13

- **Tags:** L small_L
- **Algorithm:** `F' L' U' L U L' U' L U F`
- **Pattern:** TXR / LXX / BBR
- **Result:** Case #10

### Case #14

- **Tags:** L small_L
- **Algorithm:** `l' U R' U' R l U2 (x') U' R U l'`
- **Pattern:** LXT / LXX / LBB
- **Result:** Case #17

### Case #15

- **Tags:** L small_L
- **Algorithm:** `R' F R2 B' R2' F' R2 B R'`
- **Pattern:** TXR / XXR / BBR
- **Result:** Case #14 rotated 180°

### Case #16

- **Tags:** L small_L
- **Algorithm:** `r' U' R U' R' U R U' R' U2 r`
- **Pattern:** LTR / LXX / LXR
- **Result:** Case #16 rotated counter-clockwise 90°

### Case #17

- **Tags:** L small_L
- **Algorithm:** `r U R' U R U' R' U R U2' r'`
- **Pattern:** LXR / LXX / LBR
- **Result:** Case #17 rotated clockwise 90°

## W-Shapes

### Case #33 — Basement Stairs

- **Tags:** L W_shapes
- **Algorithm:** `R U R' U R U' R' U' R' F R F'`
- **Pattern:** TXX / XXR / XBR
- **Result:** Case #36 rotated counter-clockwise 90°

### Case #34 — Upstairs

- **Tags:** L W_shapes
- **Algorithm:** `R' U' R U' R' U R U l U' R' U (x)`
- **Pattern:** XTR / XXR / BXX
- **Result:** Case #35 rotated counter-clockwise 90°

## Fish

### Case #18 — Down Boomerang

- **Tags:** L fish
- **Algorithm:** `R' U' R (y' x') R U' R' F R U l'`
- **Pattern:** LXT / XXR / BBX
- **Result:** Case #27

### Case #19 — Up Boomerang

- **Tags:** L fish
- **Algorithm:** `R U R' (x z') R' U R B' R' U' l`
- **Pattern:** TTX / XXR / LXB
- **Result:** Case #26

### Case #31 — Dot Kite

- **Tags:** L fish
- **Algorithm:** `R' U2 l R U' R' U l' U2' R`
- **Pattern:** TXX / LXX / XBR
- **Result:** Case #32

### Case #32 — Stripe Kite

- **Tags:** L fish
- **Algorithm:** `F R U' R' U' R U R' F'`
- **Pattern:** XXR / XXR / BBX
- **Result:** Case #48


## Knight Moves

### Case #26

- **Tags:** - knight_move
- **Algorithm:** `R' F R U l' U' l F U' F`
- **Pattern:** LTT / XXX / BBX
- **Result:** Case #19 rotated clockwise 90°

### Case #27

- **Tags:** - knight_move
- **Algorithm:** `(x') R U' R' F' R U R' (x y) R' U R`
- **Pattern:** TTX / XXX / LBB
- **Result:** Case #18

### Case #28

- **Tags:** - knight_move
- **Algorithm:** `L F L' R U R' U' L F' L'`
- **Pattern:** LTX / XXX / BBR
- **Result:** Case #22

### Case #29

- **Tags:** - knight_move
- **Algorithm:** `L' B' L R' U' R U L' B L`
- **Pattern:** TTR / XXX / LBX
- **Result:** Case #23 rotated 180°


## Awkward Shapes

### Case #39

- **Tags:** L awkward_shape
- **Algorithm:** `B' R B' R2' U R U R' U' R B2`
- **Pattern:** XTX / XXR / LXR
- **Result:** Case #47

### Case #40

- **Tags:** L awkward_shape
- **Algorithm:** `R2' U R' B' R U' R2' U l U l'`
- **Pattern:** XTX / LXX / LXR
- **Result:** Case #48

### Case #41

- **Tags:** L awkward_shape
- **Algorithm:** `R U R' U R U2' R' F R U R' U' F'`
- **Pattern:** TXT / XXR / XBX
- **Result:** Case #38 rotated counter-clockwise 90°

### Case #42

- **Tags:** L awkward_shape
- **Algorithm:** `R' U' R U' R' U2 R F R U R' U' F'`
- **Pattern:** XTX / XXR / BXB
- **Result:** Case #41

## Corners Correct, Edges Flipped

### Case #53 — Helipad

- **Tags:** - corners_correct
- **Algorithm:** `R U R' U' r R' U R U' r'`
- **Pattern:** XTX / XXX / XBX
- **Result:** Case #54 rotated 180°

### Case #54 — Chipped Teeth

- **Tags:** L corners_correct
- **Algorithm:** `r R' U R r' U2 r R' U R r'`
- **Pattern:** XTX / LXX / XXX
- **Result:** Case #54 rotated clockwise 90°

## OCLL / Solves

### Case #49 — T-Shirt


- **Tags:** + solves
- **Algorithm:** `R U2' R2' U' R2 U' R2' U2 R`
- **Pattern:** LXT / XXX / LXB
- **Result:** Case #49 (same)

### Case #50 — Car

- **Tags:** + solves
- **Algorithm:** `R U R' U R U' R' U R U2' R'`
- **Pattern:** LXR / XXX / LXR
- **Result:** Case #50 rotated clockwise 90°

### Case #51 — Spaceship

- **Tags:** + solves
- **Algorithm:** `R U R' U R U2' R'`
- **Pattern:** TXR / XXX / XXB
- **Result:** Case #52

### Case #52 — Kickboxer

- **Tags:** + solves
- **Algorithm:** `R U2 R' U' R U' R'`
- **Pattern:** LXX / XXX / BXR
- **Result:** Case #51

### Case #55 — Bird Flip

- **Tags:** + solves
- **Algorithm:** `R2' D R' U2 R D' R' U2 R'`
- **Pattern:** XXX / XXX / BXB
- **Result:** Case #57 rotated 180°

### Case #56 — Bull

- **Tags:** + solves
- **Algorithm:** `l' U' L U R U' r' F`
- **Pattern:** XXT / XXX / XXB
- **Result:** Case #57 rotated counter-clockwise 90°

### Case #57 — Dino

- **Tags:** + solves
- **Algorithm:** `l' U' L' U R U' l U`
- **Pattern:** XXT / XXX / LXX
- **Result:** Case #56
