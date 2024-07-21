PROGRAM Palindrome;
{ Determine if the number input is a palindrome. }

VAR
	x, y, digit: integer;
	isPalindrome: boolean;
BEGIN
	write('Number: ');
	read(x);
	y := 0;
	IF x < 0 THEN
		writeln(false)
	ELSE BEGIN
		isPalindrome := x < 10;
		digit := 0;
		WHILE NOT isPalindrome AND (x > 0) DO BEGIN
			isPalindrome := x = y;
			y := y * 10 + digit;
			isPalindrome := isPalindrome OR (x = y);
			digit := x mod 10;
			x := x div 10
		END;
		writeln(isPalindrome)
	END
END.



