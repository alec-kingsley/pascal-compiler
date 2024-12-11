PROGRAM quadratic (input, output);
	VAR
		a, b, c, discriminant, re, im : real;
	BEGIN
		read(a,b,c);
		IF (a = 0) AND (b = 0)
			THEN writeln('The equation is degenerate')
		ELSE IF a = 0
			THEN writeln('Single root is', -c/b)
		ELSE IF c = 0
			THEN writeln('The roots are', -b/a, 'and', 0)
		ELSE
			BEGIN
				re := - b / (2 * a);
				discriminant := sqr(b) - 4 * a * c;
				im := sqrt(abs(discriminant)) / (2 * a);
				IF discriminant >= 0
					THEN writeln('The roots are', re + im, 
					                       'and', re - im)
					ELSE writeln('The roots are complex',
					                re, '+I*', im,
					                'and', re, '-I*', im)
			END
	END. { quadratic }

{
	Input:
		0    0    7
		0   10    2
		2    3    0
		1    5    6
		1    1    1
	
	Output:
		The equation is degenerate
		Single root is -0.2000000
		The roots are -1.5000000 and 0
		The roots are -2.0000000 and -3.000000
		The roots are complex -5.000000 +I* 0.866025
											and -5.000000 +I* 0.866025
}
