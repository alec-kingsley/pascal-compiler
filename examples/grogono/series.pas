PROGRAM series (input, output);
	VAR
		termcount : integer;
		sum, limit : real;
	BEGIN
		termcount := 0;
		sum := 0;
		read(limit);
		REPEAT
			termcount := termcount + 1;
			sum := sum + 1/termcount
		UNTIL sum > limit;
		write(termcount)
	END. { series }

{
	Input:
	     5
		  10

	Output:
         83
		  12367
}
