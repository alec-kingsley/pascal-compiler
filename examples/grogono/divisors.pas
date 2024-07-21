PROGRAM divisors (input, output);
  VAR
    number, divisor : integer;
  BEGIN
    REPEAT
      read(number);
      IF number > 0
        THEN
	  BEGIN
	    writeln('The divisors of', number, 'are:');
	    FOR divisor := 2 TO number DO
	     IF number MOD divisor = 0
	       THEN writeln(divisor)
	  END
    UNTIL number <= 0
 END. { divisors }

