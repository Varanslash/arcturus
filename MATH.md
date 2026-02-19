# ARCTURUS TO BYTECODE SPEC
## ARITHMETIC OPS
### =    assign

PUSH <value>
STORE <variable>

### +=   addassign

PUSH <number>
; this will be done for however many values are on the other side
; ex: x = 6 + 5, push 6, push 5
; ex: x += 6, same as x = x + 6, so push x, push 6
ADD <x> ; pop x values deep and add together, auto push
STORE <dest>

### -=   subassign

PUSH <number> ; same as addassign
; ex: x = 6 - 5, push 6, push 5
; ex: x -= 6, same as x = x - 6, so push x, push 6
SUB <x> ; pop x values deep and subtract all
STORE <dest>

### *=   mulassign

PUSH <number> ; same as addassign
MUL <x> ; pop x values deep and multiply all
STORE <dest>

### /=   divassign

PUSH <number>
DIV <x>
STORE <dest>

### NON ASSIGN

; all of these are the same as the assign variants, except they don't load onto the stack
; unless manually assigned via =.
; they can also be used in comparisons
+    add
-    sub
*    mul
/    div

; bit shifting
### >>   bsr
; x >> is the same as x *= 2

### <<   bsl
; x << is the same as x /= 2

## COMPARISON OPS

; most comparison ops are the same - we can use a single system
PUSH <left value>
PUSH <right value>
COMPARE <cmp> ; auto pushes true or false