PROGRAM PascalsTriangle;
{ calculate and display Pascal's Triangle }

CONST
	ROWCT = 10;
VAR
	arr: ARRAY[0..ROWCT + 1] of integer;
	i: integer;
BEGIN
	{ initialize row to all 0s }
	FOR i := 1 TO ROWCT + 1 DO
		arr[i] := 0;

	{ create first row (1) }
	arr[ROWCT] := 1;

	WHILE arr[0] = 0 DO BEGIN
		{ print last row and generate next one }
		FOR i := 0 TO ROWCT DO BEGIN
			IF arr[i] > 0 THEN WRITE(arr[i], ' ');
			arr[i] := arr[i] + arr[i + 1]
		END;
		WRITELN
	END
END.




