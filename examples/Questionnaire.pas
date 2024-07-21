PROGRAM Questionnaire;

VAR
	name, animal: String;
	age: Integer;

BEGIN
	writeln('Welcome to the questionnaire!');
	write('Enter your name: ');
	read(name);
	write('Enter your age: ');
	read(age);
	write('Enter an animal: ');
	read(animal);
	writeln('Hello, ', name, '! ', animal, ' is a good choice.');
	if age < 18 then
		writeln('You will be 18 in ', 18 - age, ' years.')
	else if age = 18 then
		writeln('You are 18!')
	else
		writeln('You have been an adult for ', age - 18, ' years.')
END.
