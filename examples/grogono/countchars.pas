PROGRAM countchars (input, output);
  CONST
    blank = ' ';
    comma = ',';
    period = '.';
  VAR
    charcount, blankcount, commacount, periodcount
      : integer;
    character : char;
  BEGIN
    charcount := 0;
    blankcount := 0;
    commacount := 0;
    periodcount := 0;
    WHILE NOT eof DO
      BEGIN
        read(character);
	charcount := charcount + 1;
	IF character = blank
	  THEN blankcount := blankcount + 1
	ELSE IF character = comma
	  THEN commacount := commacount + 1
	ELSE IF character = period
	  THEN periodcount := periodcount + 1
      END; { while }
    writeln(charcount, 'characters');
    writeln(blankcount, 'blanks');
    writeln(commacount, 'commas');
    writeln(periodcount, 'periods')
  END. { countchars }

