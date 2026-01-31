# Bubble Sort - sorts array [5,1,4,2,8] → [1,2,4,5,8]
# Registers: r1=base, r2=n, r3=i, r4=j, r10/r11=ptrs, r12/r13=vals, r100=const1

SET 100, 1          # constant 1
SET 2, 5            # n = 5

# Allocate array [5,1,4,2,8]
NEW 1, 1
SET 50, 5
WRITE 1, 0, 50
NEW 0, 1
ADD 10, 1, 100
SET 50, 1
WRITE 10, 0, 50
NEW 0, 1
ADD 10, 10, 100
SET 50, 4
WRITE 10, 0, 50
NEW 0, 1
ADD 10, 10, 100
SET 50, 2
WRITE 10, 0, 50
NEW 0, 1
ADD 10, 10, 100
SET 50, 8
WRITE 10, 0, 50

# Sort
SET 3, 0            # i = 0
OuterLoop:
CMP 3, 2
BEQ Done
SUB 5, 2, 100       # j_limit = n-1-i
SUB 5, 5, 3
SET 4, 0            # j = 0

InnerLoop:
CMP 4, 5
BGT NextI
BEQ NextI
ADD 10, 1, 4        # ptr_j
READ 12, 10, 0
ADD 11, 10, 100     # ptr_j+1
READ 13, 11, 0
CMP 12, 13
BLT NoSwap
BEQ NoSwap
WRITE 10, 0, 13     # swap
WRITE 11, 0, 12

NoSwap:
ADD 4, 4, 100
JMP InnerLoop

NextI:
ADD 3, 3, 100
JMP OuterLoop

Done:
SETS 255, Sorted!
TEXEC 0x5000, 255, 0
HALT
