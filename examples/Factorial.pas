PROGRAM Factorial;

VAR
	n, r: integer;
BEGIN
	write('Enter a number: ');
	read(n);
	write(n);
	r := 1;
	WHILE n > 1 DO BEGIN
		r := r * n;
		n := n - 1;
	END
	write('! = ');
	writeln(r);
END.
