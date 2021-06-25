# Async Experiment on BL602

This is an experiment to use async Rust on BL602.

It's just a very simple async executor and a `Future` which can wait for a specified amount of ticks (currently each tick is half a second).

It is deliberately simple.

It uses an not yet merged PR of the BL602-HAL - but it should work with the `main` branch, too.
