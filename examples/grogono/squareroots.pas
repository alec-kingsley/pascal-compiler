PROGRAM squareroots (input,output);
	VAR
		x : real;
	BEGIN
		REPEAT
			read(x);
			IF x >= 0
				THEN write(sqrt(x))
	ELSE write('argument error')
			UNTIL x = 0
	END.

{
Input:
		2
		3
		4
		0
}
