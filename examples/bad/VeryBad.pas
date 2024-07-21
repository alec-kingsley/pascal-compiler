PROGRAM VeryBad;

VAR
	i: integer;
	s: string;
BEGIN
	FOR s := 1 TO 7 DO
		i := 'Oops...';
	IF i THEN
		writeln('Shoulda been a boolean');
END.


