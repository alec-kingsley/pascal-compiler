PROGRAM minimax (input, output);
  VAR
    reading : boolean;
    number, minimum, maximum, count
      : integer;
  BEGIN
    reading := true;
    minimum := maxint;
    maximum := - maxint;
    count := 0;
    WHILE reading DO
      BEGIN
        read(number);
	IF number = 0
	  THEN reading := false
	  ELSE
	    BEGIN
	      count := count + 1;
	      IF number < minimum
	        THEN minimum := number
	      ELSE IF number > maximum
	        THEN maximum := number
	    END
      END; { while }
    writeln(count, 'numbers read');
    writeln('The smallest was', minimum);
    writeln('The largest was', maximum)
  END. { minimax }
