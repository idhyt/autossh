
all: passh

passh: passh.c

clean:
	-rm passh

.PHONY: all clean

lib:
	gcc -c -o passh.o passh.c
	ar rc passh.a passh.o