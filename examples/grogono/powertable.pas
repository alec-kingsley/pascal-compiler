PROGRAM powertable (input,output);
	VAR
		tablesize, base, square, cube, quad
			: integer;
	BEGIN
		read(tablesize);
		FOR base := 1 TO tablesize DO
			BEGIN
				square := sqr(base);
	cube := base * square;
	quad := sqr(square);
	writeln(base,square,cube,quad,
		1/base,1/square,1/cube,1/quad)
			END { for }
	END. { powertable }

