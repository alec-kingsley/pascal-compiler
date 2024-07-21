PROGRAM SquareRoot;

VAR
	goal, tooHigh, lowEnough, avg: integer;
BEGIN
	write('Number: ');
	read(goal);

	lowEnough := 0;
	tooHigh := goal + 1;

	WHILE tooHigh - lowEnough > 1 DO BEGIN
		avg := (tooHigh + lowEnough) div 2;
		IF avg * avg > goal THEN
			tooHigh := avg
		ELSE
			lowEnough := avg;
	END;
	writeln('sqrt(', goal, ') = ', lowEnough);
END.


