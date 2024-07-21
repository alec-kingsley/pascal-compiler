PROGRAM OverUnder;
{ 
* this program allows you to attempt to guess a random
* number generated from a seed you supply
}
VAR
	seed, secretNumber, guess: integer;
BEGIN
	write('Enter a random number to be used as a seed: ');
	read(seed); { this seed will be used for the secret number }

	secretNumber := (25173 * seed + 13849) MOD 1000 + 1;
	
	{ impossible value for secretNumber }
	guess := -1;

	WHILE guess <> secretNumber DO BEGIN
		write('Guess: ');
		read(guess);
		IF guess < secretNumber THEN
			writeln(-1) { too low }
		ELSE IF guess > secretNumber THEN
			writeln(1) { too high }
		ELSE
			writeln(0)
	END

END.
