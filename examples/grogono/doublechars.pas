PROGRAM doublechars (input, output);
  CONST
    blank = ' ';
  VAR
    oldchar, newchar : char;
  BEGIN
    oldchar := blank;
    WHILE NOT eof DO
      BEGIN
        read(newchar);
	IF (newchar <> blank) AND (oldchar = newchar)
	  THEN writeln(oldchar,newchar);
	oldchar := newchar
      END { while }
  END. { doublechar }

