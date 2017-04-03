trip
====

Finds tripcodes that contain patterns

Building
--------

    cargo build --release

Usage
-----

    trip [pattern]

trip finds 2channel-style tripcodes that contain any of the pattern given as
arguments. If a tripcode containing a pattern is found, trip will print the
password and the tripcode it generates. trip will search for tripcodes
until a key is pressed, at which point it will terminate and print the average
number of tripcodes generated per second.
