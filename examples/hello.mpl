# Hello World in MicroPerl

my $name = "World";
my $count = 5;

print "Hello, ", $name, "!\n";

# Simple loop
my $i = 0;
while ($i < $count) {
    print "Count: ", $i, "\n";
    $i++;
}

# Subroutine
sub greet($who) {
    print "Greetings, ", $who, "!\n";
}

greet("Z80");
