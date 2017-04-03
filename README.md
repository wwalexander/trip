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
password and the tripcode it generates. By default, trip will only use one
processor. The number of processors to use can be set using the PROCS
environment variable.