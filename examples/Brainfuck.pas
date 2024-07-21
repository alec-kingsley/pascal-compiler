PROGRAM Brainfuck;

VAR
	mem: array[1..3000] of char;
	code, inputText: string;
	codeIdx, memIdx, inputIdx, balance: integer;

BEGIN
	write('Code: ');
	readln(code);

	write('Input: ');
	readln(inputText);

	FOR memIdx := 1 to 3000 DO
		mem[memIdx] := chr(0);
	memIdx := 1;

	codeIdx := 1;
	inputIdx := 1;
	WHILE ord(code[codeIdx]) <> 0 DO BEGIN
		IF code[codeIdx] = '+' THEN
			mem[memIdx] := chr(ord(mem[memIdx]) + 1)
		ELSE IF code[codeIdx] = '-' THEN
			mem[memIdx] := chr(ord(mem[memIdx]) - 1)
		ELSE IF code[codeIdx] = '>' THEN
			memIdx := memIdx + 1
		ELSE IF code[codeIdx] = '<' THEN
			memIdx := memIdx - 1
		ELSE IF code[codeIdx] = '.' THEN
			write(mem[memIdx])
		ELSE IF (code[codeIdx] = ',') and (ord(inputText[inputIdx]) <> 10) THEN BEGIN
			mem[memIdx] := inputText[inputIdx];
			inputIdx := inputIdx + 1;
		END ELSE IF (code[codeIdx] = '[') and (ord(mem[memIdx]) = 0) THEN BEGIN
			balance := 1;
			WHILE balance <> 0 DO BEGIN
				codeIdx := codeIdx + 1;
				IF code[codeIdx] = '[' THEN
					balance := balance + 1
				ELSE IF code[codeIdx] = ']' THEN
					balance := balance - 1
			END
		END ELSE IF (code[codeIdx] = ']') and (ord(mem[memIdx]) <> 0) THEN BEGIN
			balance := -1;
			WHILE balance <> 0 DO BEGIN
				codeIdx := codeIdx - 1;
				IF code[codeIdx] = '[' THEN
					balance := balance + 1
				ELSE IF code[codeIdx] = ']' THEN
					balance := balance - 1
			END
		END;
		codeIdx := codeIdx + 1;
	END;

	writeln;

END.


