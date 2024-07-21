PROGRAM ForLoops;

VAR
	i: integer;
BEGIN
	writeln('Count up to 5:');
	FOR i := 1 TO 5 DO
		writeln('i: ', i);
	writeln('Count down from 5:');
	FOR i := 5 DOWNTO 1 DO
		writeln('i: ', i)
END.

