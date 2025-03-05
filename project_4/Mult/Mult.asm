// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
// The algorithm is based on repetitive addition.

@R0
D=M

@i
M=D

@prod
M=0

@RETURN
D;JEQ

(loop)
@R1
D=M

@RETURN
D;JEQ

@prod
M=D+M

@i
MD=M-1

@loop
D;JGT

(RETURN)
@prod
D=M

@R2
M=D

@END
0;JMP