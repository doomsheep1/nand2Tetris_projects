// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/4/Fill.asm

// Runs an infinite loop that listens to the keyboard input. 
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel. When no key is pressed, 
// the screen should be cleared.

(REFRESH)
@SCREEN
D=A
// calculate last screen address start
@LASTADDRESS
M=D

@8192
D=A

@LASTADDRESS
M=D+M
M=M-1
// calculate last screen address end

@i
M=D

@KBD
D=M

(loop)
@KBD
D=M

@SETBLACK
D;JNE

@LASTADDRESS
D=M
A=D
M=0

@CONT
0;JMP

(SETBLACK)
@LASTADDRESS
D=M
A=D
M=-1

(CONT)
@LASTADDRESS
M=M-1

@i
MD=M-1

@loop
D;JGT

@REFRESH
0;JMP
