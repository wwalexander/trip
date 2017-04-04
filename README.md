trip 
====

Finds tripcodes that contain patterns

![trip's current version badge](https://img.shields.io/crates/v/trip.svg)

Building
--------

    cargo build --release

Usage
-----

    trip [pattern]...

trip finds 2channel-style tripcodes that contain any of the patterns given as
arguments. If a tripcode containing a pattern is found, trip will print the
password and the tripcode it generates. By default, trip will only use one
processor. The number of processors to use can be set using the
NUMBER\_OF\_PROCESSORS environment variable.