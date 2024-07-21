PROGRAM formats (output);
  CONST
    multiplier = 10;
    finalvalue = 1000000;
  VAR
    power : integer;
  BEGIN
    power := multiplier;
    REPEAT
      writeln('*', power:1, '*');
      power := power * multiplier
    UNTIL power >= finalvalue
  END.
