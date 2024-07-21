PROGRAM Fibonacci; { prints n terms of fibonacci sequence }

VAR
	a, b, c: integer; { these are for calculations }
	n: integer; { this is for # of runs }
BEGIN
	a := 0;
	b := 1;
	write('Enter the number of terms you want: ');
	read(n);
	WHILE n > 0 DO BEGIN
		{ calculate next term }
		c := a + b;
		a := b;
		b := c;

		n := n - 1;

		if n > 0 then
			write(a, ', ')
		else
			writeln(a)
	END
	
END.



