PROGRAM findsquareroots (input, output);
	CONST
		epsilon = 1E-6;
	VAR
		number, root : real;
	BEGIN
		REPEAT
			read(number);
			IF number < 0
				THEN writeln('Argument error')
			ELSE IF number = 0
				THEN writeln(0)
			ELSE { number > 0 }
				BEGIN
					root := 1;
					REPEAT
						root := (number / root + root) / 2
					UNTIL abs(number / sqr(root) - 1) < epsilon;
					writeln(root)
				END
		UNTIL number = 0
	END. { findsquareroots }

{
	Input:
		  1 2 3 4 5 -1 0
	
	Output:
		  1.000000
			1.414214
			1.732051
			2.000000
			2.236067
			Argument error
			0
}
